use crate::binary::sender::Sender;
use crate::streaming::session::Session;
use crate::streaming::systems::system::System;
use anyhow::Result;
use iggy::error::Error;
use iggy::users::logout_user::LogoutUser;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

pub async fn handle(
    command: &LogoutUser,
    sender: &mut dyn Sender,
    session: &mut Session,
    system: Arc<RwLock<System>>,
) -> Result<(), Error> {
    debug!("session: {session}, command: {command}");
    let system = system.read().await;
    system.logout_user(session).await?;
    session.clear_user_id();
    sender.send_empty_ok_response().await?;
    Ok(())
}
