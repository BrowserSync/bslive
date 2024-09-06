import {
  createBrowserRouter,
  RouterProvider,
  useLoaderData,
} from "react-router-dom";

import "./index.css";

let router = createBrowserRouter([
  {
    path: "/",
    loader: async () => fetch('/abc'),
    Component() {
      let data = useLoaderData();
      return (
        <div>
          <p>API response from <code>/abc</code></p>
          <pre><code>{JSON.stringify(data)}</code></pre>
        </div>
      );
    },
  },
]);

export default function App() {
  return <RouterProvider router={router} fallbackElement={<p>Loading...</p>} />;
}

if (import.meta.hot) {
  import.meta.hot.dispose(() => router.dispose());
}
