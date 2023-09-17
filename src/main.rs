mod pages;
use pages::login::Login;
use pages::reporting::Reporting;
use pages::search::Search;

use std::rc::Rc;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authorized {
    pub inner: bool,
}

impl Reducible for Authorized {
    type Action = bool;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        Authorized { inner: action }.into()
    }
}

pub type AuthorizedContext = UseReducerHandle<Authorized>;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Login,
    #[at("/search")]
    Search,
    #[at("/reporting/:uid")]
    Reporting {uid: String},
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Search => html! { <Search /> },
        Route::Login => html! { <Login /> },
        Route::Reporting {uid} => html! { <Reporting study_uid={uid} /> },
        Route::NotFound => html! { <h1>{"404: Not Found"}</h1> },
    }
}

#[function_component(App)]
fn app() -> Html {
    let ctx = use_reducer(|| Authorized { inner: false });

    html! {
        <ContextProvider<AuthorizedContext> context={ctx}>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </ContextProvider<AuthorizedContext>>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
