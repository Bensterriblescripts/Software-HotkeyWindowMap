use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    NancyTitle = <View> {
        width: Fill,
        height: Fit,
        margin: 10.0,
        title = <H3> {
            text: "Nancy - Hotkey and Window Management System"
        }
    }

    NancyHeader = <View> {
        width: Fill, height: Fit,
        flow: Down,
        spacing: 10.,
        margin: {top: 0., right: 9, bottom: 0., left: 9}
        divider = <Hr> { }
        title = <H3> { }
    }

    NancyDesc = <P> {  }


    App = {{App}} {
        ui: <Window> {
            show_bg: true,
            draw_bg: {
                color: #222222
            }
            window: {
                title: "Makepad UI zoo"
            },
            caption_bar = {
                visible: true,
                margin: {left: -500},
                caption_label = { label = {text: "Makepad book UI Zoo caption bar"} },
            },

            body = <View> {
                width: Fill, height: Fill,
                flow: Down, //child components will be arranged vertically
                spacing: 10.,   //spacing between child components
                margin: 0.,     //margin around the component

                <NancyTitle> {}

                <NancyHeader> {
                    title = {text: "Intro"}
                    <NancyDesc> {
                        text: "Intro."
                    }
                    <View> {
                        width: Fill, height: Fit,
                        flow: Down,
                        <P> { text: "- Shader-based: what does that mean for how things work." }
                        <P> { text: "- Inheritance mechanisms in the DSL." }
                        <P> { text: "- Introduction to the layout system." }
                        <P> { text: "- Base theme parameters." }
                        <P> { text: "- Typographic system. Base font-size and contrast." }
                        <P> { text: "- Space constants to control denseness of the design." }
                        <P> { text: "- Transparency mechanism of the widgets. Nesting for structure." }
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

app_main!(App);