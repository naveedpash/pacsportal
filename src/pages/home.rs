use dicom::dictionary_std::tags;
use dicom::object::InMemDicomObject;
use gloo::net::http::Request;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    let studies = use_state(|| vec![]);
    let is_loaded = use_state(|| false);

    {
        let studies = studies.clone();
        let is_loaded = is_loaded.clone();
        // use_effect(move || {
        //     wasm_bindgen_futures::spawn_local(async move {
        //         let mwl_response: serde_json::Value =
        //             Request::post("http://210.56.0.36:8080/dcm4chee-arc/aets/SCHPACS2/rs/mwlitems")
        //                 .body("{}")
        //                 .unwrap()
        //                 .send()
        //                 .await
        //                 .unwrap()
        //                 .json()
        //                 .await
        //                 .unwrap();
        //     });
        // });
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_studies: Vec<serde_json::Value> = Request::get("http://210.56.0.36:8080/dcm4chee-arc/aets/SCHPACS2/rs/studies?StudyDate=20230720")
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
            },
            (),
        );
    }

    let id_filter = use_state(|| String::from(""));
    let name_filter = use_state(|| String::from(""));
    let accession_filter = use_state(|| String::from(""));
    let modality_filter = use_state(|| String::from(""));
    let description_filter = use_state(|| String::from(""));
    let source_ae_filter = use_state(|| String::from(""));
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
                // .filter(|entry| entry.description.contains(description_filter.as_str()))
                // .filter(|entry| {
                //     entry
                //         .source_ae
                //         .to_lowercase()
                //         .contains(source_ae_filter.as_str())
                // })
                .collect::<Vec<InMemDicomObject>>()
        },
        [
            (*id_filter).clone(),
            (*name_filter).clone(),
            (*accession_filter).clone(),
            (*modality_filter).clone(),
            (*description_filter).clone(),
            (*source_ae_filter).clone(),
        ],
    );
    // six node refs for the search boxes at the column headers
    let filter_node_refs = vec![
        NodeRef::default(),
        NodeRef::default(),
        NodeRef::default(),
        NodeRef::default(),
        NodeRef::default(),
        NodeRef::default(),
    ];

    let filter_callback = {
        let filter_node_refs = filter_node_refs.clone();
        Callback::from(move |_: KeyboardEvent| {
            let id = filter_node_refs[0].cast::<HtmlInputElement>();
            let name = filter_node_refs[1].cast::<HtmlInputElement>();
            let accession = filter_node_refs[2].cast::<HtmlInputElement>();
            let modality = filter_node_refs[3].cast::<HtmlInputElement>();
            let description = filter_node_refs[4].cast::<HtmlInputElement>();
            let source_ae = filter_node_refs[5].cast::<HtmlInputElement>();
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
        })
    };

    let header = html_nested! {
        <thead>
            <tr>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onkeypress={&filter_callback} ref={&filter_node_refs[0]} placeholder={"Patient ID"} /></th>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onkeypress={&filter_callback} ref={&filter_node_refs[1]} placeholder={"Name"} /></th>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onkeypress={&filter_callback} ref={&filter_node_refs[2]} placeholder={"Accession"} /></th>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onkeypress={&filter_callback} ref={&filter_node_refs[3]} placeholder={"Modality"} /></th>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onkeypress={&filter_callback} ref={&filter_node_refs[4]} placeholder={"Description"} /></th>
                <th><input type={"text"} class={classes!(String::from("w-full block"))} onkeypress={&filter_callback} ref={&filter_node_refs[5]} placeholder={"Source AE"} /></th>
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
        let studies = (*studies).clone();
        move || -> Html {
            if *is_loaded {
                html! {
                    <tbody>
                        {
                            studies.iter().map(move |entry| {
                                let id = entry.element_by_name("PatientID").unwrap().to_str().unwrap();
                                let name = entry.element_by_name("PatientName").unwrap().to_str().unwrap();
                                let accession = entry.element_by_name("AccessionNumber").unwrap().to_str().unwrap();
                                let modalities = entry.element_by_name("ModalitiesInStudy").unwrap().strings().unwrap().join(", ");
                                let description = "";
                                let source_ae = "";
                                let date = entry.element_by_name("StudyDate").unwrap().to_str().unwrap();
                                let time = entry.element_by_name("StudyTime").unwrap().to_str().unwrap();
                                html!{
                                    <tr key={id.clone().into_owned()} class={classes!(String::from("hover:bg-[#d01c25]"))}>
                                        <td>{id}</td>
                                        <td>{name}</td>
                                        <td>{accession}</td>
                                        <td>{modalities}</td>
                                        <td>{description}</td>
                                        <td>{source_ae}</td>
                                        <td>{date} {time}</td>
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
                <div class={classes!(String::from("flex items-center"))}>
                    <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} placeholder={"Select Date"}/>
                    <span class={classes!(String::from("mx-4 text-gray-500"))}>{"to"}</span>
                    <input type={"date"} class={classes!(String::from("px-2 py-1 border"))} placeholder={"Select Date"}/>
                </div>
                <div class={classes!(String::from("flex m-2"))}>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] rounded-l dark:text-white"))}>{"1d"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"3d"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"1w"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"1m"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] dark:text-white"))}>{"1y"}</button>
                    <button class={classes!(String::from("px-2 py-1 border hover:bg-[#F5CE04] hover:text-[#040404] rounded-r dark:text-white"))}>{"Any"}</button>
                </div>
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
