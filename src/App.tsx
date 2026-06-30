import { Route, Routes } from "react-router-dom";
import ListePersonnes from "./pages/ListePersonnes";
import DetailPersonne from "./pages/DetailPersonne";

export default function App() {
  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-5xl mx-auto px-4 py-3 flex items-center justify-between">
          <h1 className="text-xl font-semibold text-gray-800">Cadence</h1>
        </div>
      </header>
      <main className="max-w-5xl mx-auto px-4 py-6">
        <Routes>
          <Route path="/" element={<ListePersonnes />} />
          <Route path="/personnes/:id" element={<DetailPersonne />} />
        </Routes>
      </main>
    </div>
  );
}
