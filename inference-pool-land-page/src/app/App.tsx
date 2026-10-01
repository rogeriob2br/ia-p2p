import { useState } from "react";
import { LandingPage } from "@/app/components/LandingPage";
import { SignUp } from "@/app/components/SignUp";
import { Documentation } from "@/app/components/Documentation";
import { Dashboard } from "@/app/components/Dashboard";

type Page =
  | "landing"
  | "signup"
  | "documentation"
  | "dashboard";

export default function App() {
  const [currentPage, setCurrentPage] =
    useState<Page>("landing");

  const handleNavigate = (page: string) => {
    setCurrentPage(page as Page);
    // Scroll to top when navigating
    window.scrollTo({ top: 0, behavior: "smooth" });
  };

  return (
    <>
      {currentPage === "landing" && (
        <LandingPage onNavigate={handleNavigate} />
      )}
      {currentPage === "signup" && (
        <SignUp onNavigate={handleNavigate} />
      )}
      {currentPage === "documentation" && (
        <Documentation onNavigate={handleNavigate} />
      )}
      {currentPage === "dashboard" && (
        <Dashboard onNavigate={handleNavigate} />
      )}
    </>
  );
}