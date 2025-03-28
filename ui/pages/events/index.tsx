// import { useState, useEffect } from "react"
import useSWR from 'swr'

interface Event {
    title: string,
    start_date: string,
    end_date: string
}

const fetcher = (...args: [RequestInfo | URL, RequestInit?]) => fetch(args[0], {...args[1], credentials: 'include'}).then((res) => res.json())

export default function Page(){
    const {data, error} = useSWR<Event[]>("http://localhost:8000/events", fetcher)

    if (!data) return <p>Loading data...</p>
    if (error) return <p>error</p>

    return (
        <>
            <h1>Events</h1>
            <ul>
                {data.map((e) => (
                    <li key={e.title + e.start_date + e.end_date} className="border-solid border-radius-2 border-gray-300">
                        <div>{e.title}</div>
                        <div>{e.start_date}</div>
                        <div>to</div>
                        <div>{e.end_date}</div>
                    </li>
                ))}
            </ul>
        </>
    )
}