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
  const res = await fetch(`http://localhost:8000/items`);
  const product = await res.json();
  return product;
}

// HydrateFallback is rendered while the client loader is running
export function HydrateFallback() {
  return <div>Loading...</div>;
}

export default function Component( {loaderData} : Route.ComponentProps){
    return (
        <div className="flex flex-col items-center justify-center">
        {
            loaderData.map(function (a : any) {
                return (
                    <div className="text-center w-[200px] rounded-3x1 border border-gray-200 p-6 space-y-2">
                        <h1 className="">{a.name}</h1>
                        <p>{a.price} $</p>
                    </div>
            )})
        }
        </div>
    );
}