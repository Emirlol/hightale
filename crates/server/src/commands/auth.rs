use std::sync::Arc;

use command::{
    command,
    CommandRegistry,
};
use net::auth::ServerAuthManager;
use tokio::runtime::Handle;

pub fn register(registry: &mut CommandRegistry, auth: Arc<ServerAuthManager>, rt: Handle) {
	let auth_clone_1 = auth.clone();
	let auth_clone_2 = auth.clone();
    let rt_1 = rt.clone();
    let rt_2 = rt.clone();
    command!(registry, "auth", {
		literal "login" {
			literal "browser" executes move |ctx| {
				let sender = ctx.sender.clone();
				sender.send_message("Starting Browser Authentication...");
				let auth = auth_clone_1.clone();
                let rt = rt_1.clone();
				rt.spawn(async move {
					match auth.start_browser_flow().await {
						Ok(url) => {
							sender.send_message("========================================");
							sender.send_message("Open this URL to login:");
							sender.send_message(&format!("{}", url));
							sender.send_message("========================================");

							if webbrowser::open(&url).is_ok() {
								sender.send_message("(Browser opened automatically)");
							}
						}
						Err(e) => sender.send_error(&format!("Failed to start auth flow: {}", e)),
					}
				});

				Ok(())
			}
		}

		literal "status" executes move |ctx| {
			let auth = auth_clone_2.clone();
			let sender = ctx.sender.clone();
            let rt = rt_2.clone();
			rt.spawn(async move {
				if auth.get_identity_token().await.is_some() {
					sender.send_message("Authenticated (Token present)");
				} else {
					sender.send_message("Status: Offline / Unauthenticated");
				}
			});
			Ok(())
		}
	});
}
