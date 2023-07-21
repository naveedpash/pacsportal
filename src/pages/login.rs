use gloo::net::http;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[function_component(Login)]
fn login() -> Html {
    let username_node_ref = use_node_ref();
    let password_node_ref = use_node_ref();
    let error_message = use_state(|| String::default());

    let submit_callback = {
        let username_node_ref = username_node_ref.clone();
        let password_node_ref = password_node_ref.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let error_message = error_message.clone();
            let username = username_node_ref
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();
            let password = password_node_ref
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();
            let credentials = Credentials { username, password };

            spawn_local(async move {
                let response = http::Request::post("http://localhost:4040/login")
                    .header("Content-Type", "application/json")
                    .json(&credentials)
                    .unwrap()
                    .send()
                    .await;
                match response {
                    Ok(_) => println!("Successfully authenticated!"),
                    Err(e) => error_message.set(e.to_string()),
                }
            });
        })
    };

    html! {
        <>
            <ybc::Container classes={classes!("is-centered")}>
                <h1 classes={classes!("title", "has-text-centered")}>{"South City Hospital Radiology Portal"}</h1>
                <h2 classes={classes!("subtitle", "has-text-centered")}>{"Login"}</h2>

                <form action="">
                    <ybc::Field>
                        <label classes={classes!("label")}>{"Username"}</label>
                        <ybc::Control classes={classes!("has-icons-left")}>
                            <input classes={classes!("input")} type={"text"} placeholder={"Username"} ref={&username_node_ref} />
                            <span classes={classes!("icon", "is-small", "is-left")}>
                                <Icon icon_id={IconId::FontAwesomeSolidCircleUser} />
                            </span>
                        </ybc::Control>
                    </ybc::Field>
                    <ybc::Field>
                        <label classes={classes!("label")}>{"Username"}</label>
                        <ybc::Control>
                            <input classes={classes!("input")} type={"text"} placeholder={"Password"} ref={&password_node_ref}/>
                            <span classes={classes!("icon", "is-small", "is-left")}>
                                <Icon icon_id={IconId::FontAwesomeSolidKey} />
                            </span>
                        </ybc::Control>
                    </ybc::Field>
                </form>
            </ybc::Container>
        </>
    }
}
