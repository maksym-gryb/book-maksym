import type { Route } from "./+types/items";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "New React Router App" },
    { name: "description", content: "Welcome to React Router!" },
  ];
}

export async function clientLoader({
  params,
}: Route.ClientLoaderArgs) {
  const res = await fetch(`http://localhost:8000/events`);
  const product = await res.json();
  return product;
}

// HydrateFallback is rendered while the client loader is running
export function HydrateFallback() {
  return <div>Loading...</div>;
}

export default function Component( {loaderData} : Route.ComponentProps){
    return (
        <div className="flex flex-col items-center justify-center space-y-8">
        {
            loaderData.map(function (a : any) {
                return (
                    <div className="text-center w-[200px] border border-gray-200 p-6 space-y-3">
                        <p className="text-3xl capitalize">{a.title}</p>
                        <hr />
                        <p className="rounded-2xl border py-2">Start Date: {a.start_date}</p>
                        <p className="rounded-2xl border py-2">End Date: {a.end_date}</p>
                    </div>
            )})
        }
        </div>
    );
}