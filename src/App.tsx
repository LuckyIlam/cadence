import { Route, Routes } from "react-router-dom";
import Nav from "./components/Nav";
import Activites from "./pages/Activites";
import DetailActivite from "./pages/DetailActivite";
import DetailPersonne from "./pages/DetailPersonne";
import ListePersonnes from "./pages/ListePersonnes";
import PlanningPage from "./pages/PlanningPage";

export default function App() {
  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-5xl mx-auto px-4 py-3 flex items-center justify-between">
          <h1 className="text-xl font-semibold text-gray-800">Cadence</h1>
          <Nav />
        </div>
      </header>
      <main className="max-w-5xl mx-auto px-4 py-6">
        <Routes>
          <Route path="/" element={<ListePersonnes />} />
          <Route path="/personnes/:id" element={<DetailPersonne />} />
          <Route path="/activites" element={<Activites />} />
          <Route path="/activites/:id" element={<DetailActivite />} />
          <Route path="/planning" element={<PlanningPage />} />
          <Route path="/planning/:personneId" element={<PlanningPage />} />
        </Routes>
      </main>
    </div>
  );
}
