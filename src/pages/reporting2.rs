use data_encoding::BASE64;
use pdf_writer::{Content, Finish, Name, PdfWriter, Rect, Ref, Str};
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

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
                let mut writer = PdfWriter::new();
                let catalog_id = Ref::new(1);
                let page_tree_id = Ref::new(2);
                let page_id = Ref::new(3);
                let font_id = Ref::new(4);
                let content_id = Ref::new(5);
                let font_name = Name(b"F1");

                writer.catalog(catalog_id).pages(page_tree_id);
                writer.pages(page_tree_id).kids([page_id]).count(1);

                let mut page = writer.page(page_id);
                page.media_box(Rect {
                    x1: 0.0,
                    y1: 0.0,
                    x2: 595.0,
                    y2: 842.0,
                });
                page.parent(page_tree_id);
                page.contents(content_id);
                page.resources().fonts().pair(font_name, font_id);
                page.finish();

                writer.type1_font(font_id).base_font(Name(b"Helvetica"));
                let mut content = Content::new();
                content.begin_text();
                content.set_font(font_name, 14.0);
                content.next_line(108.0, 734.0);
                content.show(Str((*report).as_bytes()));
                content.end_text();
                writer.stream(content_id, &content.finish());

                let buf: Vec<u8> = writer.finish();
                let encoded_buf = BASE64.encode(&buf);
                let mut current_report = String::from("data:application/pdf;base64,");
                current_report.push_str(&encoded_buf);
                pdf_report.set(current_report);
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
