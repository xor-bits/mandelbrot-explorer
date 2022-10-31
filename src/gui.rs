use srs2dge::prelude::*;

//

#[derive(Debug, Widget)]
pub struct Root {
    debug_text: Text<'static>,

    #[gui(core)]
    core: WidgetCore,
}

//

impl Root {
    pub fn set_text<P: Into<FormatStringPart<'static>>>(&mut self, p: P) {
        self.debug_text.text(
            FormatString::builder()
                .with_init(Format::new(Color::BLACK, 0, 24.0))
                .with(p),
        );
    }
}

//

pub fn root() -> Root {
    let stylesheet = stylesheet! {
        => {
            text_align: TextAlign::top_left(),
        }
    };

    Root::build(stylesheet.get_default(), &stylesheet)
}
