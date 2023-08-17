mod pages;
use pages::home::Home;
use pages::reporting::Reporting;

use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    // #[at("/login")]
    // Login,
    #[at("/reporting")]
    Reporting,
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        // Route::Login => html! { <Login /> }
        Route::Reporting => html! { <Reporting /> },
        Route::NotFound => html! { <h1>{"404: Not Found"}</h1> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
