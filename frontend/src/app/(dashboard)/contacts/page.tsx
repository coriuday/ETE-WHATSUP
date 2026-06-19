"use client";

import { useEffect, useState } from "react";
import {
  Users,
  Search,
  Plus,
  Upload,
  Trash2,
  Tag,
  ChevronLeft,
  ChevronRight,
  Filter,
  X,
  AlertCircle,
  RefreshCw,
  Download
} from "lucide-react";
import toast from "react-hot-toast";

interface ContactItem {
  id: string;
  phone_number: string;
  first_name: string | null;
  last_name: string | null;
  email: string | null;
  tags: string[];
  wa_status: string;
  source: string;
  created_at: string;
}

export default function Contacts() {
  const [contacts, setContacts] = useState<ContactItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Search & Filter
  const [search, setSearch] = useState("");
  const [selectedTag, setSelectedTag] = useState("");
  const [allTags, setAllTags] = useState<string[]>([]);

  // Pagination
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [totalCount, setTotalCount] = useState(0);

  // Dialog Modals
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [isImportOpen, setIsImportOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  // Add Contact Form State
  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [phoneNumber, setPhoneNumber] = useState("");
  const [email, setEmail] = useState("");
  const [tagsInput, setTagsInput] = useState("");

  // Import State
  const [csvFile, setCsvFile] = useState<File | null>(null);
  const [importing, setImporting] = useState(false);

  const fetchContacts = async () => {
    setLoading(true);
    setError("");
    try {
      const { api } = await import("@/lib/api");
      const res = await api.get("/contacts", {
        params: {
          page,
          search: search || undefined,
          tags: selectedTag || undefined,
        }
      });
      // Backend returns: { success, data: { data: [...], pagination: { total, page, total_pages, ... } } }
      const responseData = res.data.data;
      setContacts(responseData.data || []);
      setTotalPages(responseData.pagination?.total_pages || 1);
      setTotalCount(responseData.pagination?.total || 0);

      // Derive all tags dynamically
      const tags = new Set<string>();
      (responseData.data || []).forEach((c: ContactItem) => c.tags?.forEach((t: string) => tags.add(t)));
      if (tags.size > 0) setAllTags(Array.from(tags));
    } catch (e: any) {
      const msg = e.response?.data?.error?.message || "Failed to load contacts";
      setError(typeof msg === "string" ? msg : "Failed to load contacts");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchContacts();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page, selectedTag]);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setPage(1);
    fetchContacts();
  };

  const handleAddContact = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    const formattedTags = tagsInput.split(",").map(t => t.trim()).filter(t => t !== "");

    try {
      const { api } = await import("@/lib/api");
      await api.post("/contacts", {
        phone_number: phoneNumber,
        first_name: firstName || null,
        last_name: lastName || null,
        email: email || null,
        tags: formattedTags.length > 0 ? formattedTags : undefined,
      });

      toast.success("Contact added successfully!");
      setIsAddOpen(false);
      resetAddForm();
      fetchContacts();
    } catch (e: any) {
      const msg = e.response?.data?.error?.message || "Failed to add contact";
      toast.error(typeof msg === "string" ? msg : "Failed to add contact");
    } finally {
      setSubmitting(false);
    }
  };

  const resetAddForm = () => {
    setFirstName("");
    setLastName("");
    setPhoneNumber("");
    setEmail("");
    setTagsInput("");
  };

  const handleImportContacts = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!csvFile) {
      toast.error("Please choose a CSV file to import");
      return;
    }

    setImporting(true);
    const formData = new FormData();
    formData.append("file", csvFile);

    try {
      const { api } = await import("@/lib/api");
      const res = await api.post("/contacts/import", formData, {
        headers: { "Content-Type": "multipart/form-data" },
      });

      const jobId = res.data.data?.job_id;
      toast.success(jobId ? `Import started! Job ID: ${jobId.slice(0, 8)}...` : "Bulk import triggered!");
      setIsImportOpen(false);
      setCsvFile(null);
      // Wait a bit then refresh
      setTimeout(() => fetchContacts(), 3000);
    } catch (e: any) {
      const msg = e.response?.data?.error?.message || "Import failed";
      toast.error(typeof msg === "string" ? msg : "Import failed");
    } finally {
      setImporting(false);
    }
  };

  const handleDeleteContact = async (id: string) => {
    if (!confirm("Are you sure you want to delete this contact?")) return;

    try {
      const { api } = await import("@/lib/api");
      await api.delete(`/contacts/${id}`);
      toast.success("Contact deleted");
      fetchContacts();
    } catch (e: any) {
      const msg = e.response?.data?.error?.message || "Failed to delete contact";
      toast.error(typeof msg === "string" ? msg : "Failed to delete");
    }
  };

  return (
    <div className="space-y-6">
      {/* Title Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-white/5 pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
            <Users className="w-6 h-6 text-primary" /> Contact Database
          </h1>
          <p className="text-muted-foreground text-sm">Create segments, import spreadsheets, and view user profiles</p>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={() => setIsImportOpen(true)}
            className="px-4 py-2 text-xs font-semibold rounded-xl bg-white/5 border border-white/10 text-white hover:bg-white/10 hover-scale flex items-center gap-1.5"
          >
            <Upload className="w-4 h-4" /> Import CSV
          </button>
          <button
            onClick={() => setIsAddOpen(true)}
            className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 hover-scale flex items-center gap-1.5"
          >
            <Plus className="w-4 h-4" /> Add Contact
          </button>
        </div>
      </div>

      {/* Filter and Search Bar */}
      <div className="glass-panel p-4 rounded-2xl flex flex-col md:flex-row gap-4 items-center justify-between">
        <form onSubmit={handleSearchSubmit} className="relative w-full md:w-96">
          <Search className="absolute left-3.5 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground/60" />
          <input
            type="text"
            placeholder="Search by name, phone, or email..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-10 pr-4 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent text-sm placeholder-white/20"
          />
        </form>

        <div className="flex flex-wrap items-center gap-3 w-full md:w-auto">
          {allTags.length > 0 && (
            <div className="relative">
              <select
                value={selectedTag}
                onChange={(e) => { setSelectedTag(e.target.value); setPage(1); }}
                className="appearance-none bg-white/5 border border-white/10 text-sm font-semibold rounded-xl px-4 py-2.5 pr-8 focus:outline-none focus:ring-2 focus:ring-primary text-white cursor-pointer"
              >
                <option value="" className="bg-slate-900">Filter by Tag</option>
                {allTags.map(tag => (
                  <option key={tag} value={tag} className="bg-slate-900">{tag}</option>
                ))}
              </select>
              <Filter className="absolute right-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground/60 pointer-events-none" />
            </div>
          )}

          {selectedTag && (
            <button onClick={() => setSelectedTag("")} className="p-2.5 rounded-xl bg-white/5 border border-white/10 text-muted-foreground hover:text-white hover:bg-white/10">
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>

      {/* Contacts Table */}
      <div className="glass-panel rounded-2xl border border-white/5 overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="border-b border-white/5 bg-white/2">
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Name</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Phone</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Email</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Tags</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Status</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/5">
              {loading ? (
                <tr>
                  <td colSpan={6} className="px-6 py-10 text-center">
                    <div className="w-8 h-8 border-4 border-primary/20 border-t-primary rounded-full animate-spin mx-auto" />
                  </td>
                </tr>
              ) : error ? (
                <tr>
                  <td colSpan={6} className="px-6 py-10 text-center">
                    <AlertCircle className="w-6 h-6 text-rose-400 mx-auto mb-2" />
                    <p className="text-muted-foreground text-sm mb-3">{error}</p>
                    <button onClick={fetchContacts} className="px-4 py-1.5 text-xs font-semibold rounded-xl bg-white/5 border border-white/10 text-white hover:bg-white/10">
                      <RefreshCw className="w-3.5 h-3.5 inline mr-1" /> Retry
                    </button>
                  </td>
                </tr>
              ) : contacts.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-6 py-10 text-center text-muted-foreground text-sm">
                    No contacts found. Add some contacts or import a list to start sending.
                  </td>
                </tr>
              ) : (
                contacts.map((contact) => (
                  <tr key={contact.id} className="hover:bg-white/2 transition-colors">
                    <td className="px-6 py-4 text-sm font-semibold text-white">
                      {contact.first_name || contact.last_name
                        ? `${contact.first_name || ""} ${contact.last_name || ""}`.trim()
                        : "Unnamed Contact"}
                    </td>
                    <td className="px-6 py-4 text-sm text-muted-foreground font-mono">{contact.phone_number}</td>
                    <td className="px-6 py-4 text-sm text-muted-foreground">{contact.email || "—"}</td>
                    <td className="px-6 py-4">
                      <div className="flex flex-wrap gap-1.5">
                        {contact.tags?.map(tag => (
                          <span key={tag} className="inline-flex items-center gap-1 text-[10px] font-bold px-2 py-0.5 rounded-full bg-primary/10 text-primary border border-primary/20">
                            <Tag className="w-2.5 h-2.5" /> {tag}
                          </span>
                        ))}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <span className={`inline-block text-[9px] font-bold px-2 py-0.5 rounded-full uppercase ${
                        contact.wa_status === "active" ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20" :
                        "bg-red-500/10 text-red-400 border border-red-500/20"
                      }`}>
                        {contact.wa_status}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-right">
                      <button
                        onClick={() => handleDeleteContact(contact.id)}
                        className="p-1.5 rounded-lg border border-white/5 text-muted-foreground hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {/* Pagination Bar */}
        {totalPages > 1 && (
          <div className="px-6 py-4 border-t border-white/5 flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              Showing page {page} of {totalPages} ({totalCount} total contacts)
            </span>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setPage(p => Math.max(1, p - 1))}
                disabled={page === 1}
                className="p-1.5 rounded-lg border border-white/10 text-muted-foreground hover:text-white disabled:opacity-50 disabled:pointer-events-none"
              >
                <ChevronLeft className="w-4 h-4" />
              </button>
              <button
                onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                disabled={page === totalPages}
                className="p-1.5 rounded-lg border border-white/10 text-muted-foreground hover:text-white disabled:opacity-50 disabled:pointer-events-none"
              >
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Modal: Add Contact Dialog */}
      {isAddOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div className="fixed inset-0 bg-slate-950/65 backdrop-blur-sm" onClick={() => setIsAddOpen(false)} />
          <div className="glass-panel w-full max-w-lg rounded-2xl border border-white/10 p-6 z-10 shadow-2xl relative">
            <button onClick={() => setIsAddOpen(false)} className="absolute right-4 top-4 text-muted-foreground hover:text-white">
              <X className="w-5 h-5" />
            </button>
            <h2 className="text-lg font-bold text-white mb-4">Add New Contact</h2>

            <form onSubmit={handleAddContact} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-muted-foreground mb-1.5">First Name</label>
                  <input type="text" value={firstName} onChange={(e) => setFirstName(e.target.value)}
                    className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white" />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Last Name</label>
                  <input type="text" value={lastName} onChange={(e) => setLastName(e.target.value)}
                    className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white" />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Phone Number (with Country Code)</label>
                <input type="text" required placeholder="+919876543210" value={phoneNumber} onChange={(e) => setPhoneNumber(e.target.value)}
                  className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white" />
              </div>

              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Email Address</label>
                <input type="email" placeholder="contact@example.com" value={email} onChange={(e) => setEmail(e.target.value)}
                  className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white" />
              </div>

              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Tags (comma separated)</label>
                <input type="text" placeholder="Leads, Customer, VIP" value={tagsInput} onChange={(e) => setTagsInput(e.target.value)}
                  className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white" />
              </div>

              <div className="flex items-center justify-end gap-3 pt-4">
                <button type="button" onClick={() => setIsAddOpen(false)}
                  className="px-4 py-2 text-xs font-semibold rounded-xl bg-white/5 border border-white/10 text-white hover:bg-white/10">Cancel</button>
                <button type="submit" disabled={submitting}
                  className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 disabled:opacity-50 flex items-center gap-1.5">
                  {submitting ? <div className="w-4 h-4 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin" /> : "Add Contact"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal: Import CSV Dialog */}
      {isImportOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div className="fixed inset-0 bg-slate-950/65 backdrop-blur-sm" onClick={() => setIsImportOpen(false)} />
          <div className="glass-panel w-full max-w-md rounded-2xl border border-white/10 p-6 z-10 shadow-2xl relative">
            <button onClick={() => setIsImportOpen(false)} className="absolute right-4 top-4 text-muted-foreground hover:text-white">
              <X className="w-5 h-5" />
            </button>
            <h2 className="text-lg font-bold text-white mb-2">Import Spreadsheet</h2>
            <p className="text-muted-foreground text-xs mb-4">Choose a CSV file containing columns: phone_number, first_name, last_name, email, tags.</p>

            <form onSubmit={handleImportContacts} className="space-y-4">
              <div className="p-8 border border-dashed border-white/15 rounded-2xl flex flex-col items-center justify-center bg-white/2 hover:bg-white/4 cursor-pointer transition-colors relative">
                <input
                  type="file"
                  accept=".csv"
                  onChange={(e) => setCsvFile(e.target.files?.[0] || null)}
                  className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                />
                <Download className="w-8 h-8 text-primary mb-3" />
                <span className="text-xs font-bold text-white">{csvFile ? csvFile.name : "Choose CSV File"}</span>
                <span className="text-[10px] text-muted-foreground mt-1">Files up to 10MB supported</span>
              </div>

              <div className="flex items-center justify-end gap-3">
                <button type="button" onClick={() => setIsImportOpen(false)}
                  className="px-4 py-2 text-xs font-semibold rounded-xl bg-white/5 border border-white/10 text-white hover:bg-white/10">Cancel</button>
                <button type="submit" disabled={importing}
                  className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 disabled:opacity-50 flex items-center gap-1.5">
                  {importing ? <div className="w-4 h-4 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin" /> : "Import File"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
