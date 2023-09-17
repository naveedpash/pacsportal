use chrono::Local;
use dicom::{
    core::{DataElement, VR, DicomValue, value::DataSetSequence, Length, smallvec::smallvec},
    dictionary_std::{tags, uids},
    object::InMemDicomObject,
};
use gloo::{console::log, net::http::Request};
use uuid::Uuid;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_router::prelude::use_navigator;

use crate::Route;

#[derive(Properties, PartialEq)]
pub struct ReportProps {
    pub study_uid: String,
}

#[function_component(Reporting)]
pub fn reporting(props: &ReportProps) -> Html {
    let report = use_state(|| String::from("")); //Textual report
    let report_node_ref = NodeRef::default();
    let is_retrieving = use_state(|| true);
    let retrieving_status = use_state(|| String::from("Loading..."));
    let study_details = use_state(|| InMemDicomObject::new_empty());
    let navigator = use_navigator().unwrap();

    use_effect({
        let study_details = study_details.clone();
        let is_retrieving = is_retrieving.clone();
        let retrieving_status = retrieving_status.clone();
        let study_uid = props.study_uid.clone();
        move || {
            wasm_bindgen_futures::spawn_local(async move {
                let include_fields = "&includefield=StudyID&includefield=PatientBirthDate&includefield=PatientSex&includefield=Manufacturer";
                let fetched_details = Request::get(&format!(
                    "http://210.56.0.36:8080/dcm4chee-arc/aets/SCHPACS2/rs/studies?StudyInstanceUID={}{}",
                    study_uid, include_fields
                ))
                .send()
                .await;
                match fetched_details {
                    Ok(res) => {
                        if res.status() != 200 {
                            retrieving_status.set(format!("The server sent back an error: {}. Please report this to your system administrator.", res.status()));
                        }
                        let res_json = res.json::<Vec<serde_json::Value>>().await;
                        match res_json {
                            Ok(data) => {
                                let fetched_data: Vec<InMemDicomObject> = data.iter().map(|series| dicom_json::from_value(series.clone()).unwrap()).collect();
                                study_details.set(fetched_data[0].clone()); // because we QIDO'd a single StudyInstanceUID, we will get only one result
                                is_retrieving.set(false);
                            },
                            Err(_) => retrieving_status.set(format!("Unable to parse data from server. Please report this to your system administrator.")),
                        }
                    }
                    Err(_) => {
                        retrieving_status.set(String::from("Unable to reach the server. Please try again later or contact your system administrator."));
                    }
                };
            })
        }
    });

    let onsubmit = {
        let report = report.clone();
        let study_details = study_details.clone();
        let study_uid = props.study_uid.clone();
        let navigator = navigator.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let mut sr = InMemDicomObject::from_element_iter([
                DataElement::new(tags::SOP_CLASS_UID, VR::UI, uids::BASIC_TEXT_SR_STORAGE),
                DataElement::new(
                    tags::SOP_INSTANCE_UID,
                    VR::UI,
                    format!("2.25.{}", Uuid::new_v4()),
                ),
                study_details.get(tags::STUDY_DATE).unwrap().to_owned(),
                study_details.get(tags::STUDY_TIME).unwrap().to_owned(),
                // TODO: Need to modify this form so that the report can be back dated
                DataElement::new(tags::CONTENT_DATE, VR::DA, Local::now().date_naive().format("%Y%m%d").to_string()),
                DataElement::new(tags::CONTENT_TIME, VR::TM, Local::now().date_naive().format("%H%M%S").to_string()),
                study_details.get(tags::ACCESSION_NUMBER).unwrap().to_owned(),
                DataElement::new(tags::MODALITY, VR::CS, "SR"),
                study_details.get(tags::MANUFACTURER).unwrap().to_owned(),
                study_details.get(tags::REFERRING_PHYSICIAN_NAME).unwrap().to_owned(), // handle unwrap() errors with if let
                study_details.get(tags::PATIENT_NAME).unwrap().to_owned(),
                study_details.get(tags::PATIENT_ID).unwrap().to_owned(),
                study_details.get(tags::PATIENT_BIRTH_DATE).unwrap().to_owned(), // handle unwrap() errors with if let
                study_details.get(tags::PATIENT_SEX).unwrap().to_owned(),
                DataElement::new(
                    tags::STUDY_INSTANCE_UID,
                    VR::UI,
                    study_uid.clone()
                    // "1.2.392.200036.9116.6.18.10562196.1467.20230724090543953.1.74",
                ),
                DataElement::new(tags::SERIES_INSTANCE_UID, VR::UI, format!("2.25.{}", Uuid::new_v4())), // .to_string()?
                study_details.get(tags::STUDY_ID).unwrap().to_owned(),
                DataElement::new(tags::SERIES_NUMBER, VR::IS, "1"),
                DataElement::new(tags::INSTANCE_NUMBER, VR::IS, "1"),
                DataElement::new(tags::VERIFYING_OBSERVER_SEQUENCE, VR::SQ, DicomValue::Sequence(DataSetSequence::new(smallvec![
                    InMemDicomObject::from_element_iter([
                        DataElement::new(tags::VERIFYING_ORGANIZATION, VR::LO, "South City Hospital"),
                        DataElement::new(tags::VERIFICATION_DATE_TIME, VR::DT, Local::now().naive_local().format("%Y%m%d%H%M%S").to_string()),
                        DataElement::new(tags::VERIFYING_OBSERVER_NAME, VR::PN, "DR WASAY JILANI"),
                        DataElement::new(tags::VERIFYING_OBSERVER_IDENTIFICATION_CODE_SEQUENCE, VR::SQ, DicomValue::Sequence(DataSetSequence::empty()))
                    ])
                ], Length::UNDEFINED))),
                DataElement::new(tags::COMPLETION_FLAG, VR::CS, "COMPLETE"),
                DataElement::new(tags::VERIFICATION_FLAG, VR::CS, "VERIFIED"),
                
                DataElement::new(tags::VALUE_TYPE, VR::CS, "TEXT"),
                DataElement::new(tags::TEXT_VALUE, VR::UT, (*report).clone()),
            ]);

            let report_text = InMemDicomObject::from_element_iter([
                DataElement::new(tags::RELATIONSHIP_TYPE, VR::CS, "CONTAINS"),
                DataElement::new(tags::VALUE_TYPE, VR::CS, "TEXT"),
                DataElement::new(tags::TEXT_VALUE, VR::UT, (*report).clone()),
                // DataElement::new(tags::CONCEPT_NAME_CODE_SEQUENCE, VR::SQ, "SOMETHING_ELSE"), // TODO
                DataElement::new(tags::CONTINUITY_OF_CONTENT, VR::CS, "SEPARATE"),
            ]);

            let contents = DataElement::new(tags::CONTENT_SEQUENCE, VR::SQ, DicomValue::Sequence(DataSetSequence::new(
                smallvec![
                    report_text,
                ],
                Length::UNDEFINED
            )));

            sr.put(contents);

            let mut request_body = String::from("\r\n--myboundary");
            request_body.push_str("\r\nContent-Type: application/dicom+json\r\n\r\n");
            request_body.push_str("[");
            request_body.push_str(&dicom_json::to_string(sr).unwrap());
            request_body.push_str("]");
            request_body.push_str("\r\n--myboundary--");

            wasm_bindgen_futures::spawn_local(async move {
                let result = Request::post(
                    "http://210.56.0.36:8080/dcm4chee-arc/aets/SCHPACS2/rs/studies",
                )
                .header(
                    "Content-Type",
                    "multipart/related; type=\"application/dicom+json\"; boundary=myboundary",
                )
                .body(request_body)
                .unwrap()
                .send()
                .await
                .unwrap();

                log!(result.ok());
            });
            navigator.replace(&Route::Search);
        })
    };

    let onchange = {
        let report_node_ref = report_node_ref.clone();
        Callback::from(move |_| {
            let input = report_node_ref.cast::<HtmlTextAreaElement>();
            if let Some(input) = input {
                report.set(input.value());
            }
        })
    };

    let body = {
        let is_retrieving = is_retrieving.clone();
        let retrieving_status = retrieving_status.clone();
        let study_details = study_details.clone();
        move || -> Html {
            if !*is_retrieving {
                html! {
                    <form {onsubmit}>
            <div class="grid gap-6 mb-6 md:grid-cols-2">
                <div>
                    <div class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"Patient ID"}</div>
                    <div class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                        {study_details.get(tags::PATIENT_ID).unwrap().to_str().unwrap()}
                    </div>
                </div>
                <div>
                    <div class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"Patient ID"}</div>
                    <div class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                    {study_details.get(tags::PATIENT_NAME).unwrap().to_str().unwrap()}
                    </div>
                </div>
                <div>
                    <div class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"Patient ID"}</div>
                    <div class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                    {study_details.get(tags::PATIENT_SEX).unwrap().to_str().unwrap()}
                    </div>
                </div>
                <div>
                    <div class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"Patient ID"}</div>
                    <div class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                    {study_details.get(tags::STUDY_DATE).unwrap().to_date().unwrap().to_naive_date().unwrap().format("%Y-%m-%d").to_string()}
                    </div>
                </div>
                <div>
                    <div class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"Patient ID"}</div>
                    <div class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                    {study_details.get(tags::STUDY_TIME).unwrap().to_time().unwrap().to_naive_time().unwrap().format("%H:%M:%S").to_string()}
                    </div>
                </div>
                <div>
                    <div class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"Patient ID"}</div>
                    <div class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                    {study_details.get(tags::ACCESSION_NUMBER).unwrap().to_str().unwrap()}
                    </div>
                </div>
                <div>
                    <div class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"Patient ID"}</div>
                    <div class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                    {study_details.get(tags::MODALITIES_IN_STUDY).unwrap().strings().unwrap().join(", ")}
                    </div>
                </div>
                <div>
                    <div class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"Report:"}</div>
                    <textarea class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" onchange={&onchange} ref={&report_node_ref} required={true} placeholder={"Type your report here..."}></textarea>
                </div>
            </div>
            <button type="submit" class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm w-full sm:w-auto px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">{"Submit"}</button>
        </form>
                }
            } else {
                html! {<div>{(*retrieving_status).clone()}</div>}
            }
        }

    };

    html! {
        {body()}
    }
}
