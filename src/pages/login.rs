use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{AuthorizedContext, Route};

#[derive(Debug, Clone, PartialEq, Eq /*Serialize, Deserialize*/)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[function_component(Login)]
pub fn login() -> Html {
    let username_node_ref = use_node_ref();
    let password_node_ref = use_node_ref();
    let is_error = use_state(|| false);
    let navigator = use_navigator().unwrap();
    let auth_ctx = use_context::<AuthorizedContext>().unwrap();

    let onsubmit = {
        let username_node_ref = username_node_ref.clone();
        let password_node_ref = password_node_ref.clone();
        let is_error = is_error.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let username = username_node_ref
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();
            let password = password_node_ref
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();
            let entered_credentials = Credentials { username, password };
            let doctor_credentials = Credentials {
                username: "root".into(),
                password: "root123".into(),
            };
            let radiologist_credentials = Credentials {
                username: "radiologist".into(),
                password: "ultimate_radiologist".into(),
            };
            is_error.set(false);

            if entered_credentials == doctor_credentials
                || entered_credentials == radiologist_credentials
            {
                if entered_credentials == radiologist_credentials {
                    auth_ctx.dispatch(true);
                }
                navigator.replace(&Route::Search);
            } else {
                is_error.set(true);
            }
        })
    };

    html! {
        <div class="flex min-h-full flex-col justify-center px-6 py-12 lg:px-8">
            <div class="sm:mx-auto sm:w-full sm:max-w-sm">
                <img class="mx-auto h-30 w-auto" src="assets/sch_logo.png" alt="South City Hospital" />
                <h2 class="text-center text-2xl font-bold leading-9 tracking-tight text-gray-900">{"Radiology Department"}</h2>
            </div>

            <div class="mt-10 sm:mx-auto sm:w-full sm:max-w-sm">
                <form class="space-y-3" {onsubmit}>
                    <div>
                        <label for="username" class="block text-sm font-medium leading-6 text-gray-900">{"Username"}</label>
                        <div class="mt-1">
                            <input id="username" name="username" type="username" autocomplete="username" required={true} ref={&username_node_ref} class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6" />
                        </div>
                    </div>

                    <div>
                        <label for="password" class="block text-sm font-medium leading-6 text-gray-900">{"Password"}</label>
                        <div class="mt-1">
                            <input id="password" name="password" type="password" autocomplete="current-password" required={true} ref={&password_node_ref} class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6" />
                        </div>
                    </div>

                    <div>
                        <button type="submit" class="flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600">{"Login"}</button>
                    </div>
                    if *is_error {
                        <p>{"Incorrect username and password."}</p>
                    }
                </form>
            </div>
        </div>
    }
}
