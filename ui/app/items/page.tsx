


export default async function(){
    const r = await fetch("http://localhost:8000/events");
    const j = await r.json();

    return <>
        <h1>hello world</h1>
        <ul>
            {j.map((e) => (
                <li>{e.title} :: {e.start_date} to {e.end_date}</li>
            ))}
        </ul>
    </>
}