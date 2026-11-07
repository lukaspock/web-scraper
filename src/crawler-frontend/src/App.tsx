import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import "./App.css";

type SearchResult = { url: string; title?: string };

function App() {
    const [searchInput, setSearchInput] = useState("");
    const [submittedValue, setSubmitedValue] = useState("");

    const { data, isLoading, error } = useQuery({
        queryKey: ["search", submittedValue],
        queryFn: async () => {
            const res = await fetch(
                `http://localhost:3000/search?text=${encodeURIComponent(submittedValue)}`,
            );
            console.log(res);
            if (!res.ok) throw new Error("Fehler bei der Suche");
            return res.json();
        },
        enabled: submittedValue !== "",
    });

    return (
        <>
            <div id="center">
                <h1>Welcome to Ruggle!</h1>
                <input
                    type="text"
                    value={searchInput}
                    onChange={(e) => setSearchInput(e.target.value)}
                ></input>
                <button
                    type="button"
                    onClick={() => setSubmitedValue(searchInput)}
                >
                    Search
                </button>
            </div>

            <div id="center">
                {data?.map((d: SearchResult) => (
                    <div key={d.url}>
                        {d.title && <h1>{d.title}</h1>}
                        <a href={d.url}>{d.url}</a>
                    </div>
                ))}
            </div>
        </>
    );
}

export default App;
