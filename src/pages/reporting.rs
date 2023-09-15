use chrono::{NaiveDate, NaiveTime, Local};
use dicom::{
    core::{DataElement, VR, DicomValue, value::DataSetSequence},
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
    patient_birth_date: NaiveDate,
    patient_sex: String,
    study_date: NaiveDate,
    study_time: NaiveTime,
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
    let study_details = use_state(|| Vec::<InMemDicomObject>::new());

    use_effect({
        let study_details = study_details.clone();
        let is_retrieving = is_retrieving.clone();
        let retrieving_status = retrieving_status.clone();
        move || {
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_details = Request::get(&format!(
                    "http://210.56.0.36:8080/dcm4chee-arc/aets/SCHPACS2/rs/studies/{}/metadata",
                    props.study_uid
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
                                let fetched_data: Vec<InMemDicomObject> = data.iter().map(|series| dicom_json::from_value(*series).unwrap()).collect();
                                study_details.set(fetched_data);
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
        move |_| {
            let seq = DataSetSequence::empty();
            let sr = InMemDicomObject::from_element_iter([
                DataElement::new(tags::SOP_CLASS_UID, VR::UI, uids::BASIC_TEXT_SR_STORAGE),
                DataElement::new(
                    tags::SOP_INSTANCE_UID,
                    VR::UI,
                    format!("2.25.{}", Uuid::new_v4()),
                ),
                DataElement::new(tags::PATIENT_NAME, VR::PN, props.patient_name),
                DataElement::new(tags::PATIENT_ID, VR::LO, props.patient_id),
                DataElement::new(
                    tags::STUDY_INSTANCE_UID,
                    VR::UI,
                    "1.2.392.200036.9116.6.18.10562196.1467.20230724090543953.1.74",
                ),
                DataElement::new(tags::MODALITY, VR::CS, "SR"),
                DataElement::new(tags::SERIES_INSTANCE_UID, VR::UI, format!("2.25.{}", Uuid::new_v4())), // .to_string()?
                DataElement::new(tags::SERIES_NUMBER, VR::IS, "1"),
                DataElement::new(tags::INSTANCE_NUMBER, VR::IS, "1"),
                // TODO: Need to modify this form so that the report can be back dated
                DataElement::new(tags::CONTENT_DATE, VR::DA, Local::now().date_naive().format("%Y%m%d").to_string()),
                DataElement::new(tags::CONTENT_TIME, VR::TM, Local::now().date_naive().format("%H%M%S").to_string()),
                DataElement::new(tags::VALUE_TYPE, VR::CS, "TEXT"),
                DataElement::new(tags::TEXT_VALUE, VR::UT, "THIS IS A TEST REPORT."),
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
