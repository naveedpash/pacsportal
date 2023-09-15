use dicom::{
    core::{DataElement, VR},
    dictionary_std::{tags, uids},
    object::InMemDicomObject,
};
use gloo::{console::log, net::http::Request};
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ReportProps {
    pub patient_id: String,
    pub patient_name: String,
    pub accession_number: String,
    pub modality: String,
    pub date: String,
    pub time: String,
}

#[function_component(Reporting)]
pub fn reporting() -> Html {
    let report = use_state(|| String::from("")); //Textual report
    let pdf_report = use_state(|| String::from("")); //BASE64 encoded string
    let report_node_ref = NodeRef::default();

    use_effect_with_deps(
        {
            let report = report.clone();
            let pdf_report = pdf_report.clone();
            move |_| {
                let sr = InMemDicomObject::from_element_iter([
                    DataElement::new(tags::SOP_CLASS_UID, VR::UI, uids::BASIC_TEXT_SR_STORAGE),
                    DataElement::new(tags::SOP_INSTANCE_UID, VR::UI, "12345"),
                    DataElement::new(tags::PATIENT_NAME, VR::PN, "MRS.^NASREEN^SHAH"),
                    DataElement::new(tags::PATIENT_ID, VR::LO, "2021/022045"),
                    DataElement::new(
                        tags::STUDY_INSTANCE_UID,
                        VR::UI,
                        "1.2.392.200036.9116.6.18.10562196.1467.20230724090543953.1.74",
                    ),
                    DataElement::new(tags::MODALITY, VR::CS, "SR"),
                    DataElement::new(tags::SERIES_INSTANCE_UID, VR::UI, "123456"),
                    DataElement::new(tags::SERIES_NUMBER, VR::IS, "1"),
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
        },
        (*report).clone(),
    );

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
            <object name={"PDF Report"} type={"application/pdf"} data={(*pdf_report).clone()} class={classes!(String::from("w-full block h-screen"))}></object>
        </div>
    }
}
