use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use std::str::FromStr;
use strum::IntoEnumIterator;
use thaw::{
    Button, ButtonAppearance, Combobox, ComboboxOption, ConfigProvider, Divider, Field, Flex,
    FlexGap, FlexJustify, Input, Layout, LayoutHeader, Link, Space, Spinner, SpinnerSize, Toast,
    ToastBody, ToastIntent, ToastOptions, ToastTitle, ToasterInjection, ToasterProvider,
};

use crate::{core::types::MangaSource, server::add_manga};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/manga-tracker.css" />

        // sets the document title
        <Title text="Welcome to Leptos" />

        // content for this welcome page
        <ConfigProvider>
            <ToasterProvider>
                <Router>
                    <main>
                        <PageLayout>
                            <Routes fallback=|| "Page not found.".into_view()>
                                <Route path=StaticSegment("") view=HomePage />
                                <Route path=StaticSegment("add") view=AddMangaPage />
                            </Routes>
                        </PageLayout>
                    </main>
                </Router>
            </ToasterProvider>
        </ConfigProvider>
    }
}

#[component]
fn PageLayout(children: Children) -> impl IntoView {
    view! {
        <Layout>
            <Flex vertical=true>
                <LayoutHeader>
                    <p>"Manga Tracker"</p>
                    <Divider />
                </LayoutHeader>

                <Layout>
                    <Flex gap=FlexGap::WH(60, 60)>
                        <Flex gap=FlexGap::Large vertical=true style="padding-top:12px">
                            <Link href="/">"Home"</Link>
                            <Link href="/dashboard">"Dashboard"</Link>
                        </Flex>

                        <div style="position: absolute; left: 115px; height: 100%;">
                            <Divider vertical=true />
                        </div>
                        <Flex justify=FlexJustify::Center style="width: 100%; padding-top:12px">
                            {children()}
                        </Flex>
                    </Flex>
                </Layout>
            </Flex>
        </Layout>
    }
}

#[component]
fn AddMangaPage() -> impl IntoView {
    // state
    let manga_id = RwSignal::new("".to_owned());
    let selected_source = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);

    let toaster = ToasterInjection::expect_context();
    let on_add =
        move |_| {
            let id = manga_id.get();
            let source = selected_source
                .get()
                .map(|s| MangaSource::from_str(&s).unwrap());

            spawn_local(async move {
                is_submitting.set(true);
                let result = add_manga(id, source).await;

                match result {
                    Ok(manga) => toaster.dispatch_toast(
                        move || view! {
                            <Toast>
                                <ToastTitle>"Add Success"</ToastTitle>
                                <ToastBody>{format!("Success adding {}", manga.title)}</ToastBody>
                            </Toast>
                        },
                        ToastOptions::default().with_intent(ToastIntent::Success),
                    ),
                    Err(e) => toaster.dispatch_toast(
                        move || view! {
                            <Toast>
                                <ToastTitle>"Error"</ToastTitle>
                                <ToastBody>{e.to_string()}</ToastBody>
                            </Toast>
                        },
                        ToastOptions::default().with_intent(ToastIntent::Error),
                    ),
                }

                is_submitting.set(false);
            })
        };

    view! {
        <Title text="Add New Manga" />
        <Space vertical=true>
            <Field label="Manga ID">
                <Input value=manga_id />
            </Field>

            <Field label="Source">
                <Combobox selected_options=selected_source placeholder="Select a source">
                    {move || {
                        MangaSource::iter()
                            .map(|s| {
                                view! { <ComboboxOption value=s.to_string() text=s.to_string() /> }
                            })
                            .collect_view()
                    }}

                </Combobox>
            </Field>

            <Button appearance=ButtonAppearance::Primary on_click=on_add disabled=is_submitting>
                {move || is_submitting.get().then(|| view! { <Spinner size=SpinnerSize::Tiny /> })}
                "Add"
            </Button>
        </Space>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! { <h1>"Welcome to Manga Tracker Home Page"</h1> }
}
