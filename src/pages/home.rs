use std::collections::HashMap;

use chrono::{prelude::*, Days, Months};
use dicom::dictionary_std::tags;
use dicom::object::InMemDicomObject;
use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, HtmlInputElement};
use yew::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    let studies = use_state(|| Vec::<InMemDicomObject>::new());
    let is_loaded = use_state(|| false);
    let id_filter = use_state(|| String::from(""));
    let name_filter = use_state(|| String::from(""));
    let accession_filter = use_state(|| String::from(""));
    let modality_filter = use_state(|| String::from(""));
    let description_filter = use_state(|| String::from(""));
    let source_ae_filter = use_state(|| String::from(""));
    let start_date_filter = use_state(|| Local::now().date_naive());
    let end_date_filter = use_state(|| Local::now().date_naive());
    // let modalities_filter = use_state(|| Vec::<&str>::new());
    let modality_filters = use_state(|| {
        HashMap::from([
            ("CR", false),
            ("DR", false),
            ("CT", false),
            ("PT", false),
            ("MR", false),
            ("US", false),
            ("XA", false),
            ("NM", false),
            ("OT", false),
        ])
    });

    use_effect_with_deps(
        {
            let studies = studies.clone();
            let is_loaded = is_loaded.clone();
            let start_date_filter = start_date_filter.clone();
            let end_date_filter = end_date_filter.clone();
            let modalities_filter = modality_filters.clone();
            move |_| {
                let start_date = start_date_filter.format("%Y%m%d");
                let end_date = end_date_filter.format("%Y%m%d");
                let mut modalities = String::from("");
                modalities_filter
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
                        let patient_id =
                            study.element(tags::PATIENT_NAME).unwrap().to_str().unwrap();
                        let object = wasm_bindgen::JsValue::from(patient_id.into_owned());
                        gloo::console::log!(object);
                    });
                    studies.set(fetched_studies);
                    is_loaded.set(true);
                });
            }
        },
        [start_date_filter.clone(), end_date_filter.clone()],
    );

    let entries_to_show = use_memo(
        |_| {
            (*studies)
                .clone()
                .into_iter()
                .filter(|entry| {
                    entry
                        .element_by_name("PatientID")
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .contains(id_filter.as_str())
                })
                .filter(|entry| {
                    entry
                        .element_by_name("PatientName")
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_lowercase()
                        .contains(name_filter.as_str())
                })
                .filter(|entry| {
                    entry
                        .element_by_name("AccessionNumber")
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .contains(accession_filter.as_str())
                })
                .filter(|entry| {
                    entry
                        .element_by_name("ModalitiesInStudy")
                        .unwrap()
                        .strings()
                        .unwrap()
                        .contains(&modality_filter.as_str().to_uppercase())
                })
                .filter(|entry| {
                    if let Some(description) = entry.get(tags::STUDY_DESCRIPTION) {
                        description
                            .string()
                            .unwrap()
                            .contains(&description_filter.as_str().to_uppercase())
                    } else {
                        false
                    }
                })
                .filter(|entry| {
                    if let Some(source_ae) = entry.get(tags::SOURCE_APPLICATION_ENTITY_TITLE) {
                        source_ae
                            .string()
                            .unwrap()
                            .contains(&source_ae_filter.as_str().to_uppercase())
                    } else {
                        false
                    }
                })
                .collect::<Vec<InMemDicomObject>>()
        },
        [
            id_filter.clone(),
            name_filter.clone(),
            accession_filter.clone(),
            modality_filter.clone(),
            description_filter.clone(),
            source_ae_filter.clone(),
        ],
    );

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
        let start_date_filter = start_date_filter.clone();
        let end_date_filter = end_date_filter.clone();
        Callback::from(move |_: Event| {
            let id = filter_node_refs[0].cast::<HtmlInputElement>();
            let name = filter_node_refs[1].cast::<HtmlInputElement>();
            let accession = filter_node_refs[2].cast::<HtmlInputElement>();
            let modality = filter_node_refs[3].cast::<HtmlInputElement>();
            let description = filter_node_refs[4].cast::<HtmlInputElement>();
            let source_ae = filter_node_refs[5].cast::<HtmlInputElement>();
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
                start_date_filter.set(date);
            }
            if let Some(end_date) = end_date {
                let date =
                    NaiveDate::parse_from_str(end_date.value().as_ref(), "%Y-%m-%d").unwrap();
                end_date_filter.set(date);
            }
        })
    };

    let header = html_nested! {
        <thead>
            <tr>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onchange={&filter_callback} ref={&filter_node_refs[0]} placeholder={"Patient ID"} /></th>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onchange={&filter_callback} ref={&filter_node_refs[1]} placeholder={"Name"} /></th>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onchange={&filter_callback} ref={&filter_node_refs[2]} placeholder={"Accession"} /></th>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onchange={&filter_callback} ref={&filter_node_refs[3]} placeholder={"Modality"} /></th>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onchange={&filter_callback} ref={&filter_node_refs[4]} placeholder={"Description"} /></th>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onchange={&filter_callback} ref={&filter_node_refs[5]} placeholder={"Source AE"} /></th>
                <th>{"Date Time"}</th>
            </tr>
        </thead>
    };
    let footer = html_nested! {
        <tfoot>
            <tr>
                <th><p>{"Patient ID"}</p></th>
                <th><p>{"Name"}</p></th>
                <th><p>{"Accession"}</p></th>
                <th><p>{"Modality"}</p></th>
                <th><p>{"Description"}</p></th>
                <th><p>{"Source AE"}</p></th>
                <th><p>{"Date Time"}</p></th>
            </tr>
        </tfoot>
    };
    let body = {
        let studies = studies.clone();
        move || -> Html {
            if *is_loaded {
                html! {
                    <tbody>
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
                                html!{
                                    <tr key={id.clone().into_owned()} class={classes!(String::from("hover:bg-[#d01c25]"))}>
                                        <td>{id}</td>
                                        <td>{name}</td>
                                        <td>{accession}</td>
                                        <td>{modalities}</td>
                                        <td>{description}</td>
                                        <td>{source_ae}</td>
                                        <td>{date}{" "}{time}</td>
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

    let date_filter_callback =
        {
            let start_date_filter = start_date_filter.clone();
            let end_date_filter = end_date_filter.clone();
            Callback::from(move |e: MouseEvent| {
                end_date_filter.set(Local::now().date_naive());
                let target = e.target();
                let button = target
                    .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                    .unwrap();
                match button.name().as_str() {
                    "1D" => start_date_filter
                        .set(end_date_filter.checked_sub_days(Days::new(1)).unwrap()),
                    "3D" => start_date_filter
                        .set(end_date_filter.checked_sub_days(Days::new(3)).unwrap()),
                    "1W" => start_date_filter
                        .set(end_date_filter.checked_sub_days(Days::new(7)).unwrap()),
                    "1M" => start_date_filter
                        .set(end_date_filter.checked_sub_months(Months::new(1)).unwrap()),
                    "1Y" => start_date_filter
                        .set(end_date_filter.checked_sub_months(Months::new(12)).unwrap()),
                    &_ => start_date_filter.set(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
                }
            })
        };
    let date_query_bar = {
        let start_date_filter = start_date_filter.clone();
        let end_date_filter = end_date_filter.clone();
        let start_date = start_date_filter.format("%Y-%m-%d").to_string().to_owned();
        let end_date = end_date_filter.format("%Y-%m-%d").to_string().to_owned();
        move || -> Html {
            html! {
                <>
                    <div class={classes!(String::from("flex items-center"))}>
                        <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} value={start_date} ref={&filter_node_refs[6]} onchange={&filter_callback} />
                        <span class={classes!(String::from("mx-4 text-gray-500"))}>{"to"}</span>
                        <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} value={end_date} ref={&filter_node_refs[7]} onchange={&filter_callback} />
                    </div>
                    <div class={classes!(String::from("flex m-2"))}>
                        <button name={"1D"} onclick={&date_filter_callback} class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] rounded-l dark:text-white"))}>{"1d"}</button>
                        <button name={"3D"} onclick={&date_filter_callback} class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"3d"}</button>
                        <button name={"1W"} onclick={&date_filter_callback} class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"1w"}</button>
                        <button name={"1M"} onclick={&date_filter_callback} class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"1m"}</button>
                        <button name={"1Y"} onclick={&date_filter_callback} class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"1y"}</button>
                        <button name={"ANY"} onclick={&date_filter_callback} class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] rounded-r dark:text-white"))}>{"Any"}</button>
                    </div>
                </>
            }
        }
    };

    let modality_filter_callback = {
        let modalities_filter = modality_filters.clone();
        Callback::from(move |e: MouseEvent| {
            let target = e.target();
            let button = target
                .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                .unwrap();
            let requested_filter = button.name();
            let mut filtered_modalities = modalities_filter.clone();
            if *modalities_filter.get(requested_filter.as_str()).unwrap() {
                filtered_modalities.entry(&requested_filter);
                modalities_filter.set(filtered_modalities);
            } else {
                let mut filtered_modalities = (*modalities_filter).clone();
                filtered_modalities.append(vec![requested_filter]);
                modalities_filter.set(filtered_modalities);
            }
        })
    };
    let modality_query_bar = {
        let modalities_filter = modality_filters.clone();
        move || -> Html {
            html! {
                <div class={classes!(String::from("flex m-2"))}>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] rounded-l dark:text-white bg-[#ffd400]"))}>{"All"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"CR"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"DR"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"CT"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"PT"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"MR"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"US"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"XA"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"NM"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] rounded-r dark:text-white"))}>{"OT"}</button>
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
                    <img class={classes!(String::from("h-20 mr-3 bg-white dark:bg-gray-900"))} src="assets/sch_logo.png" alt="South City Hospital Logo" />
                    <span class={classes!(String::from("self-center text-2xl font-semibold whitespace-nowrap dark:text-white"))}>{"South City Hospital Radiology"}</span>
                </a>
            </div>
            <div class={classes!(String::from("flex items-center justify-between"))}>
                {date_query_bar()}
                {modality_query_bar()}
            </div>
        </nav>
        <div class={classes!(String::from("container mx-auto p-4 overflow-auto relative"))}>
                <table class={classes!(String::from("table-fixed w-full text-left"))}>
                    {header}
                    {body()}
                    {footer}
                </table>
        </div>
        </>
    }
}
