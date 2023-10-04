use chrono::Local;
use dicom::{
    core::{smallvec::smallvec, value::DataSetSequence, DataElement, DicomValue, Length, VR},
    dictionary_std::{tags, uids},
    object::InMemDicomObject,
};
use gloo::net::http::Request;
use uuid::Uuid;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_router::prelude::use_navigator;

#[derive(Properties, PartialEq)]
pub struct ReportProps {
    pub study_uid: String,
}

#[function_component(Reporting)]
pub fn reporting(props: &ReportProps) -> Html {
    let retrieving_status = use_state(|| String::from("Loading..."));
    let study_details = use_state(|| InMemDicomObject::new_empty());
    let report_node_ref = use_node_ref();
    let navigator = use_navigator().unwrap();

    use_effect_with_deps(
        {
            let study_uid = props.study_uid.clone();
            let study_details = study_details.clone();
            let retrieving_status = retrieving_status.clone();
            move |_| {
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
                                if res.status() == 204 {
                                    retrieving_status.set(format!("There are no search results for these search parameters. Please change your parameters and try again."));
                                } else {
                                    retrieving_status.set(format!("The server sent back an error: {}. Please report this to your system administrator.", res.status()));
                                }
                            } else {
                                let res_json = res.json::<Vec<serde_json::Value>>().await;
                                match res_json {
                                Ok(data) => {
                                    let fetched_data: Vec<InMemDicomObject> = data.iter().map(|series| dicom_json::from_value(series.clone()).unwrap()).collect();
                                    study_details.set(fetched_data[0].clone()); // because we QIDO'd a single StudyInstanceUID, we will get only one result
                                    // is_loaded.set(true);
                                    retrieving_status.set(format!(""));
                                },
                                Err(_) => retrieving_status.set(format!("Unable to parse data from server. Please report this to your system administrator.")),
                            }
                            }
                        }
                        Err(_) => {
                            retrieving_status.set(String::from("Unable to reach the server. Please try again later or contact your system administrator."));
                        }
                    };
                })
            }
        },
        (),
    );

    let onclick = {
        let study_uid = props.study_uid.clone();
        let study_details = study_details.clone();
        let report_node_ref = report_node_ref.clone();
        let navigator = navigator.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            let mut report = String::from("");
            let report_textarea = report_node_ref.cast::<HtmlTextAreaElement>();

            if let Some(report_textarea) = report_textarea {
                report = report_textarea.value();
            }

            let patient_name = study_details.get(tags::PATIENT_NAME).unwrap().string().unwrap();
            let patient_id = study_details.get(tags::PATIENT_ID).unwrap().string().unwrap();

            let sr = InMemDicomObject::from_element_iter([
                DataElement::new(tags::SOP_CLASS_UID, VR::UI, uids::BASIC_TEXT_SR_STORAGE),
                DataElement::new(
                    tags::SOP_INSTANCE_UID,
                    VR::UI,
                    format!("2.25.{}", Uuid::new_v4().simple().to_string()),
                ),
                DataElement::new(tags::PATIENT_NAME, VR::PN, patient_name),
                DataElement::new(tags::PATIENT_ID, VR::LO, patient_id),
                DataElement::new(
                    tags::STUDY_INSTANCE_UID,
                    VR::UI,
                    study_uid.clone(),
                ),
                DataElement::new(tags::MODALITY, VR::CS, "SR"),
                DataElement::new(
                    tags::SERIES_INSTANCE_UID,
                    VR::UI,
                    format!("2.25.{}", Uuid::new_v4().simple().to_string()),
                ), // .to_string()?
                DataElement::new(tags::SERIES_NUMBER, VR::IS, "1"),
                DataElement::new(tags::VALUE_TYPE, VR::CS, "TEXT"),
                DataElement::new(tags::TEXT_VALUE, VR::UT, report.clone()),
            ]);

            // let mut sr = InMemDicomObject::from_element_iter([
            //     DataElement::new(tags::SOP_CLASS_UID, VR::UI, uids::BASIC_TEXT_SR_STORAGE),
            //     DataElement::new(
            //         tags::SOP_INSTANCE_UID,
            //         VR::UI,
            //         format!("2.25.{}", Uuid::new_v4()),
            //     ),
            //     study_details.get(tags::STUDY_DATE).unwrap().to_owned(),
            //     study_details.get(tags::STUDY_TIME).unwrap().to_owned(),
            //     // TODO: Need to modify this form so that the report can be back dated
            //     DataElement::new(
            //         tags::CONTENT_DATE,
            //         VR::DA,
            //         Local::now().date_naive().format("%Y%m%d").to_string(),
            //     ),
            //     DataElement::new(
            //         tags::CONTENT_TIME,
            //         VR::TM,
            //         Local::now().naive_local().format("%H%M%S").to_string(),
            //     ),
            //     study_details
            //         .get(tags::ACCESSION_NUMBER)
            //         .unwrap()
            //         .to_owned(),
            //     DataElement::new(tags::MODALITY, VR::CS, "SR"),
            //     match study_details.get(tags::MANUFACTURER) {
            //         Some(_) => study_details.get(tags::MANUFACTURER).unwrap().to_owned(),
            //         None => DataElement::empty(tags::MANUFACTURER, VR::LO),
            //     },
            //     study_details
            //         .get(tags::REFERRING_PHYSICIAN_NAME)
            //         .unwrap()
            //         .to_owned(), // handle unwrap() errors with if let
            //     study_details.get(tags::PATIENT_NAME).unwrap().to_owned(),
            //     study_details.get(tags::PATIENT_ID).unwrap().to_owned(),
            //     match study_details.get(tags::PATIENT_BIRTH_DATE) {
            //         Some(_) => study_details.get(tags::PATIENT_BIRTH_DATE).unwrap().to_owned(),
            //         None => DataElement::empty(tags::PATIENT_BIRTH_DATE, VR::DA),
            //     },
            //     study_details
            //         .get(tags::PATIENT_BIRTH_DATE)
            //         .unwrap()
            //         .to_owned(), // handle unwrap() errors with if let
            //     study_details.get(tags::PATIENT_SEX).unwrap().to_owned(),
            //     DataElement::new(
            //         tags::STUDY_INSTANCE_UID,
            //         VR::UI,
            //         study_uid.clone(), // "1.2.392.200036.9116.6.18.10562196.1467.20230724090543953.1.74",
            //     ),
            //     DataElement::new(
            //         tags::SERIES_INSTANCE_UID,
            //         VR::UI,
            //         format!("2.25.{}", Uuid::new_v4()),
            //     ), // .to_string()?
            //     study_details.get(tags::STUDY_ID).unwrap().to_owned(),
            //     DataElement::new(tags::SERIES_NUMBER, VR::IS, "1"),
            //     DataElement::new(tags::INSTANCE_NUMBER, VR::IS, "1"),
            //     DataElement::new(
            //         tags::VERIFYING_OBSERVER_SEQUENCE,
            //         VR::SQ,
            //         DicomValue::Sequence(DataSetSequence::new(
            //             smallvec![InMemDicomObject::from_element_iter([
            //                 DataElement::new(
            //                     tags::VERIFYING_ORGANIZATION,
            //                     VR::LO,
            //                     "South City Hospital"
            //                 ),
            //                 DataElement::new(
            //                     tags::VERIFICATION_DATE_TIME,
            //                     VR::DT,
            //                     Local::now()
            //                         .naive_local()
            //                         .format("%Y%m%d%H%M%S")
            //                         .to_string()
            //                 ),
            //                 DataElement::new(
            //                     tags::VERIFYING_OBSERVER_NAME,
            //                     VR::PN,
            //                     "DR WASAY JILANI"
            //                 ),
            //                 DataElement::new(
            //                     tags::VERIFYING_OBSERVER_IDENTIFICATION_CODE_SEQUENCE,
            //                     VR::SQ,
            //                     DicomValue::Sequence(DataSetSequence::empty())
            //                 )
            //             ])],
            //             Length::UNDEFINED,
            //         )),
            //     ),
            //     DataElement::new(tags::COMPLETION_FLAG, VR::CS, "COMPLETE"),
            //     DataElement::new(tags::VERIFICATION_FLAG, VR::CS, "VERIFIED"),
            //     DataElement::new(tags::VALUE_TYPE, VR::CS, "TEXT"),
            //     DataElement::new(tags::TEXT_VALUE, VR::UT, report.clone()),
            // ]);

            // let report_text = InMemDicomObject::from_element_iter([
            //     DataElement::new(tags::RELATIONSHIP_TYPE, VR::CS, "CONTAINS"),
            //     DataElement::new(tags::VALUE_TYPE, VR::CS, "TEXT"),
            //     DataElement::new(tags::TEXT_VALUE, VR::UT, report.clone()),
            //     DataElement::new(tags::CONCEPT_NAME_CODE_SEQUENCE, VR::SQ, "SOMETHING_ELSE"), // TODO
            //     DataElement::new(tags::CONTINUITY_OF_CONTENT, VR::CS, "SEPARATE"),
            // ]);

            // let contents = DataElement::new(
            //     tags::CONTENT_SEQUENCE,
            //     VR::SQ,
            //     DicomValue::Sequence(DataSetSequence::new(
            //         smallvec![report_text,],
            //         Length::UNDEFINED,
            //     )),
            // );

            // sr.put(contents);
            let rereport = sr.element(tags::SOP_INSTANCE_UID).unwrap().to_str().unwrap();
            gloo::console::log!(wasm_bindgen::JsValue::from(rereport.into_owned()));

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

                gloo::console::log!(result.ok());
            });
            navigator.back();
        })
    };

    let body = {
        let study_details = study_details.clone();
        let navigator = navigator.clone();
        move || -> Html {
            let patient_name = study_details.get(tags::PATIENT_NAME).unwrap().to_str().unwrap().replace("^", " ").trim().to_owned();
            let modalities = study_details.get(tags::MODALITIES_IN_STUDY).unwrap().strings().unwrap().join(", ");
            let date = study_details.get(tags::STUDY_DATE).unwrap().to_date().unwrap().to_naive_date().unwrap().format("%Y-%m-%d").to_string();
            let time = study_details.get(tags::STUDY_TIME).unwrap().to_time().unwrap().to_naive_time().unwrap().format("%H:%M:%S").to_string();
            html! {
                <form class="px-6 md:px-12">
                    <div class="border-b border-gray-900/10 pb-12">
                        <h1 class="text-base font-semibold leading-7 text-gray-900">{"Reporting"}</h1>
                        <p class="mt-1 text-sm leading-6 text-gray-600">{"Please make sure you are entering the report for the correct patient and type your report below."}</p>

                        <div class="mt-10 grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-1">
                            <div class="sm:col-span-6">
                                <h3>{"Report for "}{modalities}{" of "}{patient_name}{" done on "}{date}{" at "}{time}</h3>
                            </div>
                        </div>

                        <div class="mt-10 col-span-full">
                            <label for="about" class="block text-sm font-medium leading-6 text-gray-900">{"Report"}</label>
                            <div class="mt-2">
                                <textarea ref={report_node_ref} id="report" name="about" rows="3" class="block w-full border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"></textarea>
                            </div>
                        </div>
                    </div>

                    <div class="mt-6 flex items-center justify-end gap-x-6">
                        <button onclick={
                            move |_: MouseEvent| {
                                navigator.back();
                            }
                        } type="button" class="text-sm font-semibold leading-6 text-gray-900">{"Cancel"}</button>
                        <button {onclick} type="button" class="bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600">{"Save"}</button>
                    </div>
                </form>
            }
        }
    };

    html!(
        if (*retrieving_status).clone() != String::from("") {
            <p>{(*retrieving_status).clone()}</p>
        } else {
            {body()}
        }
    )
}
