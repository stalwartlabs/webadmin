use leptos::*;
use leptos_meta::*;

#[component]
pub fn NotFound() -> impl IntoView {
    let todo = "implement not found page";
    view! {
        <Html lang="en" class="h-full"/>
        <Body class="dark:bg-slate-900 bg-gray-100 flex h-full items-center py-16"/>
        <div class="max-w-[50rem] flex flex-col mx-auto size-full">
            <header class="mb-auto flex justify-center z-50 w-full py-4">
                <nav class="px-4 sm:px-6 lg:px-8" aria-label="Global">
                    <a
                        class="flex-none text-xl font-semibold sm:text-3xl dark:text-white"
                        href="#"
                        aria-label="Brand"
                    >
                        Stalwart
                    </a>
                </nav>
            </header>

            <div class="text-center py-10 px-4 sm:px-6 lg:px-8">
                <h1 class="block text-7xl font-bold text-gray-800 sm:text-9xl dark:text-white">
                    404
                </h1>
                <h1 class="block text-2xl font-bold text-white"></h1>
                <p class="mt-3 text-gray-600 dark:text-gray-400">Oops, something went wrong.</p>
                <p class="text-gray-600 dark:text-gray-400">Sorry, we could not find your page.</p>
                <div class="mt-5 flex flex-col justify-center items-center gap-2 sm:flex-row sm:gap-3">
                    <a
                        class="w-full sm:w-auto py-3 px-4 inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:pointer-events-none dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                        href="https://github.com/htmlstreamofficial/preline/tree/main/examples/html"
                        target="_blank"
                    >
                        <svg
                            class="flex-shrink-0 size-4"
                            xmlns="http://www.w3.org/2000/svg"
                            width="16"
                            height="16"
                            fill="currentColor"
                            viewBox="0 0 16 16"
                        >
                            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z"></path>
                        </svg>
                        Get the source code
                    </a>
                    <a
                        class="w-full sm:w-auto py-3 px-4 inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent text-blue-600 hover:text-blue-800 disabled:opacity-50 disabled:pointer-events-none dark:text-blue-500 dark:hover:text-blue-400 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600"
                        href="../examples.html"
                    >
                        <svg
                            class="flex-shrink-0 size-4"
                            xmlns="http://www.w3.org/2000/svg"
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path d="m15 18-6-6 6-6"></path>
                        </svg>
                        Back to examples
                    </a>
                </div>
            </div>

            <footer class="mt-auto text-center py-5">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <p class="text-sm text-gray-500">(c) All Rights Reserved. 2022.</p>
                </div>
            </footer>
        </div>
    }
}
