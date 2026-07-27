use crate::modules::risk::application::context_builder::RiskContextBuilder;
use crate::modules::risk::application::risk_engine::evaluate_risk;
use crate::{
    app_state::AppState,
    core::{crypto::jwt, errors::app_error::AppError},
    modules::auth::{
        domain::session::session_status::SessionStatus,
        infrastructure::repositories::{
            session_repository as session_repo, user_repository::UserRepository,
        },
        interface::middleware::{
            extractor_helpers::*, policy_engine::enforce_policy, security_context::SecurityContext,
        },
    },
};
use axum::{
    extract::{ConnectInfo, State},
    http::Request,
    middleware::Next,
    response::Response,
};

use crate::modules::audit::application::{security_alerts, security_audit};
use std::net::SocketAddr;

pub async fn auth_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    // -------------------------------------------------
    // 1. Extract bearer token
    // -------------------------------------------------

    let token_from_query = req.uri().query()
        .and_then(|q| q.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            ((k == "access_token" || k == "token") && !v.is_empty()).then_some(v.to_string())
        }));
    let token_header = extract_bearer(req.headers()).ok();
    let token = token_header.or(token_from_query.as_deref()).ok_or(AppError::Unauthorized)?;

    // -------------------------------------------------
    // 2. Verify JWT
    // -------------------------------------------------

    let claims =
        jwt::verify_token(token, &state.jwt_keys.decoding).map_err(|_| AppError::Unauthorized)?;

    let jti = uuid::Uuid::parse_str(&claims.jti).map_err(|_| AppError::Unauthorized)?;

    // -------------------------------------------------
    // 3. Session lookup (source of truth)
    // -------------------------------------------------

    let session = session_repo::find_valid_session_by_jti(&state.pool, jti)
        .await
        .map_err(|_| AppError::DatabaseError)?
        .ok_or(AppError::Unauthorized)?;

    if session.status != SessionStatus::Active {
        return Err(AppError::Unauthorized);
    }

    // -------------------------------------------------
    // 4. Bind JWT to session
    // -------------------------------------------------

    if claims.sub != session.user_id {
        return Err(AppError::Unauthorized);
    }

    // -------------------------------------------------
    // 5. Load user
    // -------------------------------------------------

    let user = UserRepository::find_by_id(&state.pool, session.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // -------------------------------------------------
    // 6. Request metadata
    // -------------------------------------------------

    let ip = extract_client_ip(req.headers(), addr.ip());
    let user_agent = extract_user_agent(req.headers());

    // -------------------------------------------------
    // 7. Compute risk
    // -------------------------------------------------

    let risk_ctx = RiskContextBuilder::build(
        state.risk_signals.as_ref(),
        state.risk_history.as_ref(),
        &session,
        ip,
        &user_agent,
        jti,
    )
    .await?;

    let evaluation = evaluate_risk(&risk_ctx);

    let ctx = SecurityContext {
        user_id: session.user_id,
        role: user.user_role,
        jti,
        session_id: session.id,
        ip,
        user_agent,
        risk_score: evaluation.score.value(),
    };

    // -------------------------------------------------
    // 8. Policy enforcement
    // -------------------------------------------------

    let path = req.uri().path().to_string();

    if let Err(err) = enforce_policy(&ctx, &path) {
        let reason = match err {
            AppError::MfaRequired => "mfa_required",
            AppError::StepUpRequired => "step_up_required",
            AppError::Unauthorized => "unauthorized_or_permission_denied",
            _ => "policy_error",
        };

        // 1. Record the audit event (unchanged).
        security_audit::policy_denied(
            &state.pool,
            &state.redis,
            ctx.user_id,
            ctx.ip,
            ctx.user_agent.clone(),
            ctx.session_id,
            ctx.jti,
            &path,
            reason,
            ctx.risk_score,
        )
        .await;

        // 2. Fire an alert through the dispatcher (log + email-stub + dashboard stream).
        security_alerts::rbac_denied(
            &state.alerts,
            Some(ctx.user_id),
            Some(ctx.ip),
            &path,
            reason,
            Some(ctx.risk_score),
        )
        .await;

        return Err(err);
    }
    // -------------------------------------------------
    // 9. Touch session
    // -------------------------------------------------

    let _ = session_repo::touch_session(&state.pool, jti).await;

    // -------------------------------------------------
    // 10. Attach context
    // -------------------------------------------------

    req.extensions_mut().insert(ctx);

    // -------------------------------------------------
    // 11. Continue
    // -------------------------------------------------

    Ok(next.run(req).await)
}