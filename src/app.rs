use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use thaw::{
    ConfigProvider, Divider, Flex, FlexGap, FlexJustify, Layout, LayoutHeader, Link,
    ToasterProvider,
};

use crate::pages::dashboard::Dashboard;
use crate::pages::home::HomePage;

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
                                <Route path=StaticSegment("dashboard") view=Dashboard />
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
