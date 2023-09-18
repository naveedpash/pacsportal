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
                (String::from("DX"), false),
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
    let entries_to_show = use_state(|| Vec::<InMemDicomObject>::new());
    let is_loaded = use_state(|| false);
    // let id_filter = use_state(|| String::from(""));
    // let name_filter = use_state(|| String::from(""));
    // let accession_filter = use_state(|| String::from(""));
    // let modality_filter = use_state(|| String::from(""));
    // let description_filter = use_state(|| String::from(""));
    // let source_ae_filter = use_state(|| String::from(""));
    let fetch_filters = use_state(|| FetchFilters::new());
    let auth_ctx = use_context::<AuthorizedContext>().unwrap();
    let navigator = use_navigator().unwrap();

    let fetch_callback = {
        let studies = studies.clone();
        let entries_to_show = entries_to_show.clone();
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
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_studies: Vec<serde_json::Value> = Request::get(&format!(
                        "http://210.56.0.36:8080/dcm4chee-arc/aets/SCHPACS2/rs/studies?StudyDate={}-{}{}&includefield=StudyDescription&includefield=SourceApplicationEntityTitle",
                        start_date, end_date, modalities,
                    ))
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                let fetched_studies: Vec<InMemDicomObject> = fetched_studies
                    .iter()
                    .map(|study| dicom_json::from_value(study.clone()).unwrap())
                    .collect();
                fetched_studies.iter().for_each(|study| {
                    let patient_id = study.element(tags::PATIENT_NAME).unwrap().to_str().unwrap();
                    let object = wasm_bindgen::JsValue::from(patient_id.into_owned());
                    gloo::console::log!(object);
                });
                studies.set(fetched_studies.clone());
                entries_to_show.set(fetched_studies.clone());
                is_loaded.set(true);
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
    // .filter(|entry| {
    //     entry
    //         .element_by_name("PatientName")
    //         .unwrap()
    //         .to_str()
    //         .unwrap()
    //         .to_lowercase()
    //         .contains(name_filter.as_str())
    // })
    // .filter(|entry| {
    //     entry
    //         .element_by_name("AccessionNumber")
    //         .unwrap()
    //         .to_str()
    //         .unwrap()
    //         .contains(accession_filter.as_str())
    // })
    // .filter(|entry| {
    //     entry
    //         .element_by_name("ModalitiesInStudy")
    //         .unwrap()
    //         .strings()
    //         .unwrap()
    //         .contains(&modality_filter.as_str().to_uppercase())
    // })
    // .filter(|entry| {
    //     if let Some(description) = entry.get(tags::STUDY_DESCRIPTION) {
    //         description
    //             .string()
    //             .unwrap()
    //             .contains(&description_filter.as_str().to_uppercase())
    //     } else {
    //         false
    //     }
    // })
    // .filter(|entry| {
    //     if let Some(source_ae) = entry.get(tags::SOURCE_APPLICATION_ENTITY_TITLE) {
    //         source_ae
    //             .string()
    //             .unwrap()
    //             .contains(&source_ae_filter.as_str().to_uppercase())
    //     } else {
    //         false
    //     }
    // })
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
        let studies = studies.clone();
        let entries_to_show = entries_to_show.clone();
        Callback::from(move |_: Event| {
            let id = filter_node_refs[0]
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();
            let name = filter_node_refs[1]
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();
            let accession = filter_node_refs[2]
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();
            let modality = filter_node_refs[3]
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();
            let description = filter_node_refs[4]
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();
            let source_ae = filter_node_refs[5]
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();
            let start_date = filter_node_refs[6].cast::<HtmlInputElement>();
            let end_date = filter_node_refs[7].cast::<HtmlInputElement>();
            gloo::console::log!(wasm_bindgen::JsValue::from(id.clone()));

            let filtered_studies = (*studies)
                .clone()
                .into_iter()
                .filter(|entry| {
                    entry
                        .get(tags::PATIENT_ID)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .contains(id.as_str())
                })
                .filter(|entry| {
                    entry
                        .get(tags::PATIENT_NAME)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_lowercase()
                        .contains(name.as_str())
                })
                .filter(|entry| {
                    entry
                        .get(tags::ACCESSION_NUMBER)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .contains(accession.as_str())
                })
                .filter(|entry| {
                    entry
                        .get(tags::MODALITIES_IN_STUDY)
                        .unwrap()
                        .strings()
                        .unwrap()
                        .contains(&modality.as_str().to_uppercase())
                })
                .filter(|entry| {
                    if let Some(desc) = entry.get(tags::STUDY_DESCRIPTION) {
                        desc.string()
                            .unwrap()
                            .contains(&description.as_str().to_uppercase())
                    } else {
                        false
                    }
                })
                .filter(|entry| {
                    if let Some(sourceae) = entry.get(tags::SOURCE_APPLICATION_ENTITY_TITLE) {
                        sourceae
                            .string()
                            .unwrap()
                            .contains(&source_ae.as_str().to_uppercase())
                    } else {
                        false
                    }
                })
                .collect::<Vec<InMemDicomObject>>();
            entries_to_show.set(filtered_studies);
            // if let Some(id) = id {
            //     id_filter.set(id.value());
            // }
            // if let Some(name) = name {
            //     name_filter.set(name.value());
            // }
            // if let Some(accession) = accession {
            //     accession_filter.set(accession.value());
            // }
            // if let Some(modality) = modality {
            //     modality_filter.set(modality.value());
            // }
            // if let Some(description) = description {
            //     description_filter.set(description.value());
            // }
            // if let Some(source_ae) = source_ae {
            //     source_ae_filter.set(source_ae.value());
            // }
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
                <thead>
                    <tr class="table-row">
                        <th class="table-cell">{"Patient ID"}</th>
                        <th class="table-cell">{"Name"}</th>
                        <th class="table-cell">{"Accession"}</th>
                        <th class="table-cell">{"Modality"}</th>
                        <th class="table-cell">{"Description"}</th>
                        <th class="table-cell">{"Source AE"}</th>
                        // <th><input type={"text"} class="table-cell w-full block" onchange={&filter_callback} ref={&filter_node_refs[0]} placeholder={"Patient ID"} /></th>
                        // <th><input type={"text"} class="table-cell w-full block" onchange={&filter_callback} ref={&filter_node_refs[1]} placeholder={"Name"} /></th>
                        // <th><input type={"text"} class="table-cell w-full block" onchange={&filter_callback} ref={&filter_node_refs[2]} placeholder={"Accession"} /></th>
                        // <th><input type={"text"} class="table-cell w-full block" onchange={&filter_callback} ref={&filter_node_refs[3]} placeholder={"Modality"} /></th>
                        // <th><input type={"text"} class="table-cell w-full block" onchange={&filter_callback} ref={&filter_node_refs[4]} placeholder={"Description"} /></th>
                        // <th><input type={"text"} class="table-cell w-full block" onchange={&filter_callback} ref={&filter_node_refs[5]} placeholder={"Source AE"} /></th>
                        <th>{"Date Time"}</th>
                        {
                            if auth_ctx.inner {
                                html! {<th></th>}
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
                <tfoot>
                    <tr class="table-row">
                        <th class="table-cell"><p>{"Patient ID"}</p></th>
                        <th class="table-cell"><p>{"Name"}</p></th>
                        <th class="table-cell"><p>{"Accession"}</p></th>
                        <th class="table-cell"><p>{"Modality"}</p></th>
                        <th class="table-cell"><p>{"Description"}</p></th>
                        <th class="table-cell"><p>{"Source AE"}</p></th>
                        <th class="table-cell"><p>{"Date Time"}</p></th>
                        {
                            if auth_ctx.inner {
                                html! {<th></th>}
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
        let auth_ctx = auth_ctx.clone();
        let navigator = navigator.clone();
        let entries_to_show = entries_to_show.clone();
        move || -> Html {
            if *is_loaded {
                html! {
                    <tbody>
                        {
                            entries_to_show.iter().map(move |entry| {
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
                                let navigator = navigator.clone();
                                html!{
                                    <tr key={id.clone().into_owned()} class="table-row hover:bg-[#d01c25]">
                                        <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="table-cell block">{id}</a></td>
                                        <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="table-cell block">{name}</a></td>
                                        <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="table-cell block">{accession}</a></td>
                                        <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="table-cell block">{modalities.clone()}</a></td>
                                        <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="table-cell block">{description}</a></td>
                                        <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="table-cell block">{source_ae}</a></td>
                                        <td><a href={format!("http://210.56.0.36:3000/Viewer/{}", study_uid.clone())} target="_blank" rel="noopener noreferrer" class="table-cell block">{date}{" "}{time}</a></td>
                                        {
                                            if auth_ctx.inner && !modalities.contains("SR") {
                                                html!{
                                                    <td>
                                                        <button onclick={
                                                            move |e: MouseEvent| {
                                                                let target_uid = e.target().and_then(|t| t.dyn_into::<HtmlButtonElement>().ok()).unwrap().value();
                                                                navigator.clone().push(&Route::Reporting { uid: target_uid });
                                                        }} value={study_uid.clone().to_string()} type="submit" class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm w-full sm:w-auto px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">
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
                            }).collect::<Html>()
                        }
                    </tbody>
                }
            } else {
                html! {
                    <div>{"Loading..."}</div>
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
                "hover:bg-[#F5CE04]",
                "hover:text-[#040404]",
            ];
            html! {
                <>
                    <div class={classes!(String::from("flex items-center"))}>
                        // <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} value={start_date} max={end_date.clone()} ref={&filter_node_refs[6]} onchange={&filter_callback} />
                        <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} value={start_date} ref={&filter_node_refs[6]} onchange={&filter_callback} />
                        <span class={classes!(String::from("mx-4 text-gray-500"))}>{"to"}</span>
                        // <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} value={end_date} max={Local::now().date_naive().format("%Y-%m-%d").to_string()} ref={&filter_node_refs[7]} onchange={&filter_callback} />
                        <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} value={end_date} ref={&filter_node_refs[7]} onchange={&filter_callback} />
                    </div>
                    <div class={classes!(String::from("flex m-2"))}>
                        {
                            durations.iter().enumerate().map(|(idx, duration)| {
                                let mut needed_styles = base_styles.clone();
                                if idx == 0 {
                                    needed_styles.push("rounded-l");
                                }
                                if *duration == filter_duration {
                                    needed_styles.push("bg-[#ffd400] text-[#040404] dark:text-[#040404]");
                                } else {
                                    needed_styles.push("dark:text-white");
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
                        <button name={"ANY"} onclick={&date_filter_callback} class={classes!(base_styles, "rounded-r","dark:text-white")}>{"Any"}</button>
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
                "hover:bg-[#F5CE04]",
                "hover:text-[#040404]",
            ];
            let is_any = match fetch_filters.modalities.values().all(|v| *v == false) {
                true => "bg-[#ffd400] text-[#040404] dark:text-[#040404]",
                false => "",
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
                            needed_styles.push("bg-[#ffd400] text-[#040404] dark:text-[#040404]")
                        } else {
                            needed_styles.push("dark:text-white");
                        }
                        html!{<button name={filter.clone()} onclick={&modality_filter_callback} class={classes!(needed_styles)}>{filter.clone()}</button>}
                    }).collect::<Html>()
                }
                    <button name={"ANY"} onclick={&modality_filter_callback} class={classes!(base_styles, "rounded-r","dark:text-white", is_any)}>{"Any"}</button>
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
        <>
        <nav class={classes!(String::from("bg-white border-gray-200 dark:bg-gray-900 flex flex-wrap items-center justify-between p-4"))}>
            <div class={classes!("max-w-screen-xl","flex","flex-wrap","items-center","justify-between")}>
                <a class={classes!("flex","items-center")}>
                    <img class={classes!(String::from("h-20 mr-3 bg-white dark:bg-gray-900"))} src="assets/sch_logo.png" alt="South City Hospital" />
                    <span class={classes!(String::from("self-center text-2xl font-semibold whitespace-nowrap dark:text-white"))}>{"South City Hospital Radiology"}</span>
                </a>
            </div>
            <div class={classes!(String::from("flex items-center justify-between"))}>
                {date_query_bar()}
                {modality_query_bar()}
                <button onclick={
                    let navigator = navigator.clone();
                    move |_: MouseEvent| {
                        navigator.replace(&Route::Login);
                    }
                } type="submit" class="flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600">{"Logout"}</button>
            </div>
        </nav>
        <div class={classes!(String::from("container mx-auto p-4 overflow-auto relative"))}>
                <table class="table-fixed w-full text-left">
                    {header()}
                    {body()}
                    {footer()}
                </table>
        </div>
        </>
    }
}
