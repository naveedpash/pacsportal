use std::collections::HashMap;

use chrono::{prelude::*, Days, Months};
use dicom::dictionary_std::tags;
use dicom::object::InMemDicomObject;
use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::use_navigator;

use crate::{AuthorizedContext, Route};

#[derive(Clone, PartialEq)]
struct FetchFilters {
    start_date: NaiveDate,
    end_date: NaiveDate,
    modalities: HashMap<String, bool>,
}

impl FetchFilters {
    fn new() -> Self {
        FetchFilters {
            start_date: Local::now().date_naive(),
            end_date: Local::now().date_naive(),
            modalities: HashMap::from([
                (String::from("CR"), false),
                (String::from("DR"), false),
                (String::from("CT"), false),
                (String::from("PT"), false),
                (String::from("MR"), false),
                (String::from("US"), false),
                (String::from("XA"), false),
                (String::from("NM"), false),
                (String::from("OT"), false),
            ]),
        }
    }
}

#[function_component(Search)]
pub fn search() -> Html {
    let studies = use_state(|| Vec::<InMemDicomObject>::new());
    let is_loaded = use_state(|| false);
    let loaded_status = use_state(|| String::from("Loading..."));
    let id_filter = use_state(|| String::from(""));
    let name_filter = use_state(|| String::from(""));
    let accession_filter = use_state(|| String::from(""));
    let modality_filter = use_state(|| String::from(""));
    let description_filter = use_state(|| String::from(""));
    let source_ae_filter = use_state(|| String::from(""));
    let fetch_filters = use_state(|| FetchFilters::new());
    let auth_ctx = use_context::<AuthorizedContext>().unwrap();
    let navigator = use_navigator().unwrap();

    let fetch_callback = {
        let studies = studies.clone();
        let loaded_status = loaded_status.clone();
        let is_loaded = is_loaded.clone();
        let fetch_filters = fetch_filters.clone();
        move |_: &_| {
            let start_date = fetch_filters.start_date.format("%Y%m%d");
            let end_date = fetch_filters.end_date.format("%Y%m%d");
            let mut modalities = String::from("");
            fetch_filters
                .modalities
                .iter()
                .for_each(|(modality, is_selected)| {
                    if *is_selected {
                        modalities = format!("{}&ModalitiesInStudy={}", modalities, modality);
                    }
                });
            gloo::console::log!(wasm_bindgen::JsValue::from(modalities.clone()));
            is_loaded.set(false);
            loaded_status.set(String::from("Loading..."));
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_details = Request::get(&format!(
                    "http://210.56.0.36:8080/dcm4chee-arc/aets/SCHPACS2/rs/studies?StudyDate={}-{}{}&includefield=StudyDescription&includefield=SourceApplicationEntityTitle",
                    start_date, end_date, modalities,
                ))
                .send()
                .await;
                match fetched_details {
                    Ok(res) => {
                        if res.status() != 200 {
                            if res.status() == 204 {
                                loaded_status.set(format!("There are no search results for these search parameters. Please change your parameters and try again."));
                            } else {
                                loaded_status.set(format!("The server sent back an error: {}. Please report this to your system administrator.", res.status()));
                            }
                        } else {
                            let res_json = res.json::<Vec<serde_json::Value>>().await;
                            match res_json {
                                Ok(data) => {
                                    let fetched_data: Vec<InMemDicomObject> = data.iter().map(|series| dicom_json::from_value(series.clone()).unwrap()).collect();
                                    studies.set(fetched_data.clone());
                                    is_loaded.set(true);
                                },
                                Err(_) => loaded_status.set(format!("Unable to parse data from server. Please report this to your system administrator.")),
                            }
                        }
                    }
                    Err(_) => {
                        loaded_status.set(String::from("Unable to reach the server. Please try again later or contact your system administrator."));
                    }
                };

                studies.clone().iter().for_each(|study| {
                    let patient_id = study.element(tags::PATIENT_NAME).unwrap().to_str().unwrap();
                    let object = wasm_bindgen::JsValue::from(patient_id.into_owned());
                    gloo::console::log!(object);
                });
            });
        }
    };

    use_effect_with_deps(fetch_callback, [fetch_filters.clone()]);

    // let entries_to_show = use_memo(
    //     |_| {
    //         (*studies)
    //             .clone()
    //             .into_iter()
    //             .filter(|entry| {
    //                 entry
    //                     .element_by_name("PatientID")
    //                     .unwrap()
    //                     .to_str()
    //                     .unwrap()
    //                     .contains(id_filter.as_str())
    //             })
    //             .filter(|entry| {
    //                 entry
    //                     .element_by_name("PatientName")
    //                     .unwrap()
    //                     .to_str()
    //                     .unwrap()
    //                     .to_lowercase()
    //                     .contains(name_filter.as_str())
    //             })
    //             .filter(|entry| {
    //                 entry
    //                     .element_by_name("AccessionNumber")
    //                     .unwrap()
    //                     .to_str()
    //                     .unwrap()
    //                     .contains(accession_filter.as_str())
    //             })
    //             .filter(|entry| {
    //                 entry
    //                     .element_by_name("ModalitiesInStudy")
    //                     .unwrap()
    //                     .strings()
    //                     .unwrap()
    //                     .contains(&modality_filter.as_str().to_uppercase())
    //             })
    //             .filter(|entry| {
    //                 if let Some(description) = entry.get(tags::STUDY_DESCRIPTION) {
    //                     description
    //                         .string()
    //                         .unwrap()
    //                         .contains(&description_filter.as_str().to_uppercase())
    //                 } else {
    //                     false
    //                 }
    //             })
    //             .filter(|entry| {
    //                 if let Some(source_ae) = entry.get(tags::SOURCE_APPLICATION_ENTITY_TITLE) {
    //                     source_ae
    //                         .string()
    //                         .unwrap()
    //                         .contains(&source_ae_filter.as_str().to_uppercase())
    //                 } else {
    //                     false
    //                 }
    //             })
    //             .collect::<Vec<InMemDicomObject>>()
    //     },
    //     [
    //         id_filter.clone(),
    //         name_filter.clone(),
    //         accession_filter.clone(),
    //         modality_filter.clone(),
    //         description_filter.clone(),
    //         source_ae_filter.clone(),
    //     ],
    // );

    // eight node refs for the search boxes at the column headers and the date search boxes
    let filter_node_refs = vec![
        NodeRef::default(),
        NodeRef::default(),
        NodeRef::default(),
        NodeRef::default(),
        NodeRef::default(),
        NodeRef::default(),
        NodeRef::default(),
        NodeRef::default(),
    ];
    let filter_callback = {
        let filter_node_refs = filter_node_refs.clone();
        let fetch_filters = fetch_filters.clone();
        let id_filter = id_filter.clone();
        let name_filter = name_filter.clone();
        let accession_filter = accession_filter.clone();
        let modality_filter = modality_filter.clone();
        let description_filter = description_filter.clone();
        let source_ae_filter = source_ae_filter.clone();
        Callback::from(move |_: Event| {
            let id = filter_node_refs[0]
                .cast::<HtmlInputElement>();
            let name = filter_node_refs[1]
                .cast::<HtmlInputElement>();
            let accession = filter_node_refs[2]
                .cast::<HtmlInputElement>();
            let modality = filter_node_refs[3]
                .cast::<HtmlInputElement>();
            let description = filter_node_refs[4]
                .cast::<HtmlInputElement>();
            let source_ae = filter_node_refs[5]
                .cast::<HtmlInputElement>();
            let start_date = filter_node_refs[6].cast::<HtmlInputElement>();
            let end_date = filter_node_refs[7].cast::<HtmlInputElement>();
  
            if let Some(id) = id {
                id_filter.set(id.value());
            }
            if let Some(name) = name {
                name_filter.set(name.value());
            }
            if let Some(accession) = accession {
                accession_filter.set(accession.value());
            }
            if let Some(modality) = modality {
                modality_filter.set(modality.value());
            }
            if let Some(description) = description {
                description_filter.set(description.value());
            }
            if let Some(source_ae) = source_ae {
                source_ae_filter.set(source_ae.value());
            }
            if let Some(start_date) = start_date {
                let date =
                    NaiveDate::parse_from_str(start_date.value().as_ref(), "%Y-%m-%d").unwrap();
                let mut new_fetch_filters = (*fetch_filters).clone();
                new_fetch_filters.start_date = date;
                fetch_filters.set(new_fetch_filters);
            }
            if let Some(end_date) = end_date {
                let date =
                    NaiveDate::parse_from_str(end_date.value().as_ref(), "%Y-%m-%d").unwrap();
                let mut new_fetch_filters = (*fetch_filters).clone();
                new_fetch_filters.end_date = date;
                fetch_filters.set(new_fetch_filters);
            }
        })
    };

    let header = {
        let auth_ctx = auth_ctx.clone();
        let filter_callback = filter_callback.clone();
        let filter_node_refs = filter_node_refs.clone();
        move || -> Html {
            html! {
                <thead class="border-b font-medium dark:border-neutral-500 bg-black w-full sticky top-0">
                    <tr>
                        <th scope="col" class="px-2"><input type="text" class="peer block min-h-[auto] w-full border-0 border-b-2 bg-transparent outline-none focus:outline-none p-1 text-white" onchange={&filter_callback} ref={&filter_node_refs[0]} placeholder="Patient ID" /></th>
                        <th scope="col" class="px-2"><input type="text" class="peer block min-h-[auto] w-full border-0 border-b-2 bg-transparent outline-none focus:outline-none p-1 text-white" onchange={&filter_callback} ref={&filter_node_refs[1]} placeholder="Name" /></th>
                        <th scope="col" class="px-2"><input type="text" class="peer block min-h-[auto] w-full border-0 border-b-2 bg-transparent outline-none focus:outline-none p-1 text-white" onchange={&filter_callback} ref={&filter_node_refs[2]} placeholder="Accession" /></th>
                        <th scope="col" class="px-2"><input type="text" class="peer block min-h-[auto] w-full border-0 border-b-2 bg-transparent outline-none focus:outline-none p-1 text-white" onchange={&filter_callback} ref={&filter_node_refs[3]} placeholder="Modality" /></th>
                        <th scope="col" class="px-2"><input type="text" class="peer block min-h-[auto] w-full border-0 border-b-2 bg-transparent outline-none focus:outline-none p-1 text-white" onchange={&filter_callback} ref={&filter_node_refs[4]} placeholder="Description" /></th>
                        <th scope="col" class="px-2"><input type="text" class="peer block min-h-[auto] w-full border-0 border-b-2 bg-transparent outline-none focus:outline-none p-1 text-white" onchange={&filter_callback} ref={&filter_node_refs[5]} placeholder="Source AE" /></th>
                        <th scope="col" class="px-2 text-grey">{"Date & Time"}</th>
                        {
                            if auth_ctx.inner {
                                html! {<th scope="col" class="px-2"></th>}
                            } else {
                                html!{}
                            }
                        }
                    </tr>
                </thead>
            }
        }
    };
    let footer = {
        let auth_ctx = auth_ctx.clone();
        move || -> Html {
            html! {
                <tfoot class="border-t font-medium">
                    <tr>
                        <th scope="col" class="px-2 py-1 text-grey"><p>{"Patient ID"}</p></th>
                        <th scope="col" class="px-2 py-1 text-grey"><p>{"Name"}</p></th>
                        <th scope="col" class="px-2 py-1 text-grey"><p>{"Accession"}</p></th>
                        <th scope="col" class="px-2 py-1 text-grey"><p>{"Modality"}</p></th>
                        <th scope="col" class="px-2 py-1 text-grey"><p>{"Description"}</p></th>
                        <th scope="col" class="px-2 py-1 text-grey"><p>{"Source AE"}</p></th>
                        <th scope="col" class="px-2 py-1 text-grey"><p>{"Date & Time"}</p></th>
                        {
                            if auth_ctx.inner {
                                html! {<th scope="col" class="px-2 py-1"></th>}
                            } else {
                                html!{}
                            }
                        }
                    </tr>
                </tfoot>
            }
        }
    };
    let body = {
        let loaded_status = loaded_status.clone();
        let studies = studies.clone();
        let id_filter = id_filter.clone();
        let name_filter = name_filter.clone();
        let accession_filter = accession_filter.clone();
        let modality_filter = modality_filter.clone();
        let description_filter = description_filter.clone();
        let source_ae_filter = source_ae_filter.clone();
        let navigator = navigator.clone();
        move || -> Html {
            if *is_loaded {
                html! {
                    <tbody class="h-full overflow-y-auto">
                        {
                            studies.iter().map(move |entry| {
                                let id = entry.get(tags::PATIENT_ID).unwrap().to_str().unwrap();
                                let name = entry.get(tags::PATIENT_NAME).unwrap().to_str().unwrap().replace("^", " ").trim().to_owned();
                                let accession = entry.get(tags::ACCESSION_NUMBER).unwrap().to_str().unwrap();
                                let modalities = entry.get(tags::MODALITIES_IN_STUDY).unwrap().strings().unwrap().join(", ");
                                let description = if let Some(description) = entry.get(tags::STUDY_DESCRIPTION) {
                                    description.to_str().unwrap()
                                } else {"".into()};
                                let source_ae = if let Some(source_ae) = entry.get(tags::SOURCE_APPLICATION_ENTITY_TITLE) {
                                    source_ae.to_str().unwrap()
                                } else {"".into()};
                                let date = entry.get(tags::STUDY_DATE).unwrap().to_date().unwrap().to_naive_date().unwrap().format("%Y-%m-%d").to_string();
                                let time = entry.get(tags::STUDY_TIME).unwrap().to_time().unwrap().to_naive_time().unwrap().format("%H:%M:%S").to_string();
                                let study_uid = entry.get(tags::STUDY_INSTANCE_UID).unwrap().to_str().unwrap();
                                let to_show = id.contains(id_filter.as_str()) && name.to_lowercase().contains(name_filter.as_str()) && accession.contains(accession_filter.as_str()) && modalities.contains(&modality_filter.as_str().to_uppercase()) && description.to_lowercase().contains(description_filter.as_str()) && source_ae.contains(source_ae_filter.as_str());
                                let navigator = navigator.clone();
                                html!{
                                    if to_show {
                                        <tr key={id.clone().into_owned()} class="border-b dark:border-neutral-500 hover:bg-[#d01c25]">
                                            <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="block w-full text-white font-medium">{id}</a></td>
                                            <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="block w-full text-white">{name}</a></td>
                                            <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="block w-full text-white">{accession}</a></td>
                                            <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="block w-full text-white">{modalities.clone()}</a></td>
                                            <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="block w-full text-white">{description}</a></td>
                                            <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="block w-full text-white">{source_ae}</a></td>
                                            <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="block w-full text-white">{date}{" "}{time}</a></td>
                                            {
                                                if auth_ctx.inner && !modalities.contains("SR") {
                                                    html!{
                                                        <td>
                                                            <button onclick={
                                                                move |e: MouseEvent| {
                                                                    let target_uid = e.target().and_then(|t| t.dyn_into::<HtmlButtonElement>().ok()).unwrap().value();
                                                                    navigator.clone().push(&Route::Reporting { uid: target_uid });
                                                            }} value={study_uid.clone().to_string()} type="submit" class="inline-block px-2 py-1 bg-[#ffd400] shadow-lg text-xs font-medium">
                                                                {"Report"}
                                                            </button>
                                                        </td>
                                                    }
                                                } else {
                                                    html!{}
                                                }
                                            }
                                        </tr>
                                    }
                                    
                                }
                            }).collect::<Html>()
                        }
                    </tbody>
                }
            } else {
                html! {
                    <tbody>
                        <tr>
                            <td colspan="7" class="text-white">
                                {(*loaded_status).clone()}
                            </td>
                        </tr>
                    </tbody>
                }
            }
        }
    };

    let date_filter_callback = {
        let fetch_filters = fetch_filters.clone();
        Callback::from(move |e: MouseEvent| {
            let mut new_fetch_filters = (*fetch_filters).clone();
            new_fetch_filters.end_date = Local::now().date_naive();
            let target = e.target();
            let button = target
                .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                .unwrap();
            match button.name().as_str() {
                "1D" => {
                    new_fetch_filters.start_date = fetch_filters
                        .end_date
                        .checked_sub_days(Days::new(1))
                        .unwrap()
                }
                "3D" => {
                    new_fetch_filters.start_date = fetch_filters
                        .end_date
                        .checked_sub_days(Days::new(3))
                        .unwrap()
                }
                "1W" => {
                    new_fetch_filters.start_date = fetch_filters
                        .end_date
                        .checked_sub_days(Days::new(7))
                        .unwrap()
                }
                "1M" => {
                    new_fetch_filters.start_date = fetch_filters
                        .end_date
                        .checked_sub_months(Months::new(1))
                        .unwrap()
                }
                "1Y" => {
                    new_fetch_filters.start_date = fetch_filters
                        .end_date
                        .checked_sub_months(Months::new(12))
                        .unwrap()
                }
                &_ => new_fetch_filters.start_date = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            }
            fetch_filters.set(new_fetch_filters);
        })
    };
    let date_query_bar = {
        let fetch_filters = fetch_filters.clone();
        move || -> Html {
            let start_date = fetch_filters
                .start_date
                .format("%Y-%m-%d")
                .to_string()
                .to_owned();
            let end_date = fetch_filters
                .end_date
                .format("%Y-%m-%d")
                .to_string()
                .to_owned();
            let filter_duration = fetch_filters
                .end_date
                .signed_duration_since(fetch_filters.start_date)
                .num_days();
            let durations = vec![1, 3, 7, 30, 365];
            let base_styles = vec![
                "px-2",
                "py-1",
                "border",
                "hover:bg-yellow",
                "hover:text-black",
            ];
            html! {
                <>
                    <div class={classes!(String::from("flex items-center"))}>
                        <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} value={start_date} max={end_date.clone()} ref={&filter_node_refs[6]} onchange={&filter_callback} />
                        // <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} value={start_date} ref={&filter_node_refs[6]} onchange={&filter_callback} />
                        <span class={classes!(String::from("mx-4 text-gray-500"))}>{"to"}</span>
                        <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} value={end_date} max={Local::now().date_naive().format("%Y-%m-%d").to_string()} ref={&filter_node_refs[7]} onchange={&filter_callback} />
                        // <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} value={end_date} ref={&filter_node_refs[7]} onchange={&filter_callback} />
                    </div>
                    <div class={classes!(String::from("flex m-2"))}>
                        {
                            durations.iter().enumerate().map(|(idx, duration)| {
                                let mut needed_styles = base_styles.clone();
                                if idx == 0 {
                                    needed_styles.push("rounded-l");
                                }
                                if *duration == filter_duration {
                                    needed_styles.push("bg-[#ffd400] text-black");
                                } else {
                                    needed_styles.push("text-white dark:text-white");
                                }
                                let label = match duration {
                                    1 => "1D",
                                    3 => "3D",
                                    7 => "1W",
                                    30 => "1M",
                                    365 => "1Y",
                                    _ => "Any",
                                };
                                html!{
                                    <button name={label.clone()} onclick={&date_filter_callback} class={classes!(needed_styles)}>{label.clone()}</button>
                                }
                            }).collect::<Html>()
                        }
                        <button name={"ANY"} onclick={&date_filter_callback} class={classes!(base_styles, "rounded-r", "text-white")}>{"Any"}</button>
                    </div>
                </>
            }
        }
    };

    let modality_filter_callback = {
        let fetch_filters = fetch_filters.clone();
        Callback::from(move |e: MouseEvent| {
            let target = e.target();
            let button = target
                .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                .unwrap();
            let requested_filter = button.name();
            let mut filtered_modalities = (*fetch_filters).clone().modalities;
            if requested_filter == String::from("ANY") {
                for (_, val) in filtered_modalities.iter_mut() {
                    *val = false;
                }
            } else {
                let current_filter_status = *filtered_modalities.get(&requested_filter).unwrap();
                filtered_modalities.insert(requested_filter, !current_filter_status);
            }
            fetch_filters.set(FetchFilters {
                start_date: fetch_filters.start_date,
                end_date: fetch_filters.end_date,
                modalities: filtered_modalities,
            });
        })
    };
    let modality_query_bar = {
        let fetch_filters = fetch_filters.clone();
        move || -> Html {
            let base_styles = vec![
                "px-2",
                "py-1",
                "border",
                "hover:bg-yellow",
                "hover:text-black",
            ];
            let is_any = match fetch_filters.modalities.values().all(|v| *v == false) {
                true => "bg-[#ffd400] text-black",
                false => "text-white",
            };
            html! {
                <div class={classes!(String::from("flex m-2"))}>
                {
                    fetch_filters.modalities.clone().iter().enumerate().map(|(idx, (filter, state))| {
                        let mut needed_styles = base_styles.clone();
                        if idx == 0 {
                            needed_styles.push("rounded-l");
                        }
                        if *state {
                            needed_styles.push("bg-[#ffd400] text-black dark:text-black")
                        } else {
                            needed_styles.push("text-white dark:text-white");
                        }
                        html!{<button name={filter.clone()} onclick={&modality_filter_callback} class={classes!(needed_styles)}>{filter.clone()}</button>}
                    }).collect::<Html>()
                }
                    <button name={"ANY"} onclick={&modality_filter_callback} class={classes!(base_styles, "rounded-r", is_any)}>{"Any"}</button>
                </div>
            }
        }
    };
    /* Colors
    NATURAL GRAY #8A8887
    ALIZARIN CRIMSON #D41C24
    SUPERNOVA #F5CE04
    BLACK #040404
    */
    html! {
        <div class="h-screen bg-black flex flex-col">
            <nav class="h-1/5 flex flex-wrap items-center justify-between p-4">
                <div class="max-w-screen-xl flex flex-wrap items-center justify-between">
                    <a class="flex items-center">
                        <img class="h-20 mr-3" src="assets/sch_logo.png" alt="South City Hospital" />
                        <span class="self-center text-2xl font-semibold whitespace-nowrap text-white">{"South City Hospital Radiology"}</span>
                    </a>
                </div>
                <div class="flex items-center justify-between">
                    {date_query_bar()}
                    {modality_query_bar()}
                    <button onclick={
                        let navigator = navigator.clone();
                        move |_: MouseEvent| {
                            navigator.replace(&Route::Login);
                        }
                    } type="submit" class="flex w-full justify-center rounded-sm bg-red px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-red-600">{"Logout"}</button>
                </div>
            </nav>
            <div class="h-4/5 overflow-x-auto">
                <div class="inline-block min-w-full py-2 sm:px-6 lg:px-8">
                    <div class="container">
                        <table id="myTable" class="w-full text-left text-sm font-light">
                            {header()}
                            {body()}
                            {footer()}
                        </table>
                        // <script>
                        //     {"$(document).ready( function () {
                        //         $('#myTable').DataTable();
                        //     });"}
                        // </script>
                    </div>
                </div>
            </div>
        </div>
    }
}
