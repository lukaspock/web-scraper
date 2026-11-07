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
        <div className="app">
            <br />
            <h1>Ruggle</h1>
            <br />
            <form
                className="search"
                onSubmit={(e) => {
                    e.preventDefault();
                    setSubmitedValue(searchInput);
                }}
            >
                <input
                    type="text"
                    placeholder="Search…"
                    value={searchInput}
                    onChange={(e) => setSearchInput(e.target.value)}
                />
                <button type="submit">Search</button>
            </form>

            <div className="results">
                {isLoading && <p className="status">Searching…</p>}
                {error && <p className="status">{error.message}</p>}
                {data?.map((d: SearchResult) => (
                    <a className="result" key={d.url} href={d.url}>
                        {d.title && <h2>{d.title}</h2>}
                        <span className="url">{d.url}</span>
                    </a>
                ))}
            </div>
        </div>
    );
}

export default App;
