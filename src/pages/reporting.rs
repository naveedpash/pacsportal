use std::collections::HashMap;

use chrono::{NaiveDate, NaiveTime, Local};
use dicom::{
    core::{DataElement, VR, DicomValue, value::DataSetSequence, Length, smallvec::smallvec},
    dictionary_std::{tags, uids},
    object::InMemDicomObject,
};
use gloo::{console::log, net::http::Request};
use uuid::Uuid;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ReportProps {
    pub study_uid: String,
}

pub struct StudyDetails {
    patient_id: String,
    patient_name: String,
    patient_birth_date: String,
    patient_sex: String,
    study_date: String,
    study_time: String,
    accession_number: String,
    referring_physician_name: String,
    // study_id: String,
    modality: String,
    manufacturer: String,
}

#[function_component(Reporting)]
pub fn reporting(props: &ReportProps) -> Html {
    let report = use_state(|| String::from("")); //Textual report
    let report_node_ref = NodeRef::default();
    let is_retrieving = use_state(|| true);
    let retrieving_status = use_state(|| String::from("Loading..."));
    let study_details = use_state(|| InMemDicomObject::new_empty());

    // For Procedure Code Sequence
    // let dicom_terminology = HashMap::from([
    //     (String::from("CR"), String::from("X-ray")),
    //     (String::from("CT"), String::from("Computed Tomography")),
    //     (String::from("MR"), String::from("Magnetic Resonance")),
    //     (String::from("US"), String::from("Ultrasound")),
    //     (String::from("XA"), String::from("X-ray Angiography")),
    //     (String::from("DX"), String::from("Digital Radiography")),
    //     (String::from("PT"), String::from("Positron Emission Tomography")),
    //     (String::from("NM"), String::from("Nuclear Medicine")),
    //     (String::from("OT"), String::from("Other")),
    // ]);

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
                    Err(e) => {
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
        move |_: Event| {
            // let seq = DataSetSequence {
            //     items: smallvec![],
            //     length: Length::UNDEFINED,
            // };
            let sr = InMemDicomObject::from_element_iter([
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
                    study_uid
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
        }
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

    html! {
        <div>
            <textarea class={classes!(String::from("w-full block bg-gray-50 rounded-lg border border-gray-300 focus:ring-blue-500 focus:border-blue-500"))} onchange={&onchange} ref={&report_node_ref} placeholder={"Type your report here..."}></textarea>
        </div>
    }
}
