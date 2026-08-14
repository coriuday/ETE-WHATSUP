"use client";

import { useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { useQuery } from "@tanstack/react-query";
import { DataGrid, type GridColDef } from "@mui/x-data-grid";
import { Plus, Upload, X } from "lucide-react";
import toast from "react-hot-toast";
import { api, getErrorMessage, listContacts } from "@/lib/api";
import { PageHeader, Button, Input, Label, ErrorState, EmptyState } from "@/components/ui";

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

const PAGE_SIZE = 20;

export default function Contacts() {
  const router = useRouter();
  const [search, setSearch] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTag, setSelectedTag] = useState("");
  const [allTags, setAllTags] = useState<string[]>([]);
  const [page, setPage] = useState(1);
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [isImportOpen, setIsImportOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [phoneNumber, setPhoneNumber] = useState("");
  const [email, setEmail] = useState("");
  const [tagsInput, setTagsInput] = useState("");
  const [csvFile, setCsvFile] = useState<File | null>(null);
  const [importing, setImporting] = useState(false);

  const { data, isLoading: loading, error: queryError, refetch } = useQuery({
    queryKey: ["contacts", page, searchQuery, selectedTag],
    queryFn: async () => {
      const responseData = await listContacts({
        page,
        limit: PAGE_SIZE,
        search: searchQuery || undefined,
        tags: selectedTag || undefined,
      });
      const items = responseData?.data ?? responseData?.contacts ?? [];
      const pagination = responseData?.pagination ?? {};
      return {
        contacts: items as ContactItem[],
        totalPages: pagination.total_pages || 1,
        totalCount: pagination.total || 0,
      };
    },
  });

  const contacts = data?.contacts ?? [];
  const totalCount = data?.totalCount ?? 0;
  const error = queryError ? getErrorMessage(queryError, "Failed to load contacts") : "";

  useEffect(() => {
    const tags = new Set<string>();
    contacts.forEach((c) => c.tags?.forEach((t) => tags.add(t)));
    if (tags.size > 0) setAllTags(Array.from(tags));
  }, [contacts]);

  const columns: GridColDef<ContactItem>[] = useMemo(
    () => [
      {
        field: "name",
        headerName: "Name",
        flex: 1,
        minWidth: 140,
        valueGetter: (_value, row) =>
          `${row.first_name || ""} ${row.last_name || ""}`.trim() || "Unnamed",
      },
      { field: "phone_number", headerName: "Phone", flex: 1, minWidth: 140 },
      { field: "email", headerName: "Email", flex: 1, minWidth: 160 },
      {
        field: "tags",
        headerName: "Tags",
        flex: 1,
        minWidth: 120,
        valueGetter: (_value, row) => (row.tags || []).join(", "),
      },
      { field: "wa_status", headerName: "Status", width: 110 },
    ],
    []
  );

  const handleAddContact = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    const formattedTags = tagsInput.split(",").map((t) => t.trim()).filter(Boolean);
    try {
      await api.post("/contacts", {
        phone_number: phoneNumber,
        first_name: firstName || null,
        last_name: lastName || null,
        email: email || null,
        tags: formattedTags.length > 0 ? formattedTags : undefined,
      });
      toast.success("Contact added successfully!");
      setIsAddOpen(false);
      setFirstName("");
      setLastName("");
      setPhoneNumber("");
      setEmail("");
      setTagsInput("");
      refetch();
    } catch (err) {
      toast.error(getErrorMessage(err, "Failed to add contact"));
    } finally {
      setSubmitting(false);
    }
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
      const res = await api.post("/contacts/import", formData, {
        headers: { "Content-Type": "multipart/form-data" },
      });
      const jobId = res.data.data?.job_id;
      toast.success(jobId ? `Import started! Job ID: ${jobId.slice(0, 8)}...` : "Bulk import triggered!");
      setIsImportOpen(false);
      setCsvFile(null);
      setTimeout(() => refetch(), 3000);
    } catch (e) {
      toast.error(getErrorMessage(e, "Import failed"));
    } finally {
      setImporting(false);
    }
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="Contacts"
        description="Search, import, and open profiles."
        actions={
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => setIsImportOpen(true)}>
              <Upload className="h-4 w-4" /> Import CSV
            </Button>
            <Button onClick={() => setIsAddOpen(true)}>
              <Plus className="h-4 w-4" /> Add contact
            </Button>
          </div>
        }
      />

      <form
        onSubmit={(e) => {
          e.preventDefault();
          setPage(1);
          setSearchQuery(search);
        }}
        className="flex flex-wrap items-center gap-2"
      >
        <Input
          className="max-w-sm"
          placeholder="Search name, phone, email"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <Button type="submit" variant="outline">
          Search
        </Button>
        {allTags.length > 0 && (
          <select
            value={selectedTag}
            onChange={(e) => {
              setSelectedTag(e.target.value);
              setPage(1);
            }}
            className="h-9 rounded-lg border border-input bg-background px-3 text-sm"
          >
            <option value="">All tags</option>
            {allTags.map((tag) => (
              <option key={tag} value={tag}>
                {tag}
              </option>
            ))}
          </select>
        )}
      </form>

      {error ? (
        <ErrorState message={error} onRetry={() => refetch()} />
      ) : !loading && contacts.length === 0 ? (
        <EmptyState title="No contacts" action={<Button onClick={() => setIsAddOpen(true)}>Add contact</Button>} />
      ) : (
        <div className="h-[520px] w-full rounded-xl border border-border bg-card">
          <DataGrid
            rows={contacts}
            columns={columns}
            loading={loading}
            paginationMode="server"
            rowCount={totalCount}
            paginationModel={{ page: page - 1, pageSize: PAGE_SIZE }}
            onPaginationModelChange={(model) => setPage(model.page + 1)}
            pageSizeOptions={[PAGE_SIZE]}
            disableRowSelectionOnClick
            onRowClick={(params) => router.push(`/contacts/${params.id}`)}
            sx={{
              border: "none",
              "& .MuiDataGrid-cell:focus": { outline: "none" },
            }}
          />
        </div>
      )}

      {isAddOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div className="fixed inset-0 bg-foreground/20" onClick={() => setIsAddOpen(false)} />
          <div className="relative z-10 w-full max-w-lg rounded-xl border border-border bg-card p-6">
            <button className="absolute right-4 top-4 text-muted-foreground" onClick={() => setIsAddOpen(false)}>
              <X className="h-4 w-4" />
            </button>
            <h2 className="mb-4 text-lg font-semibold">Add contact</h2>
            <form onSubmit={handleAddContact} className="space-y-3">
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label>First name</Label>
                  <Input value={firstName} onChange={(e) => setFirstName(e.target.value)} />
                </div>
                <div className="space-y-1">
                  <Label>Last name</Label>
                  <Input value={lastName} onChange={(e) => setLastName(e.target.value)} />
                </div>
              </div>
              <div className="space-y-1">
                <Label>Phone</Label>
                <Input required placeholder="+919876543210" value={phoneNumber} onChange={(e) => setPhoneNumber(e.target.value)} />
              </div>
              <div className="space-y-1">
                <Label>Email</Label>
                <Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} />
              </div>
              <div className="space-y-1">
                <Label>Tags (comma separated)</Label>
                <Input value={tagsInput} onChange={(e) => setTagsInput(e.target.value)} />
              </div>
              <div className="flex justify-end gap-2 pt-2">
                <Button type="button" variant="outline" onClick={() => setIsAddOpen(false)}>
                  Cancel
                </Button>
                <Button type="submit" disabled={submitting}>
                  Add contact
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}

      {isImportOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div className="fixed inset-0 bg-foreground/20" onClick={() => setIsImportOpen(false)} />
          <div className="relative z-10 w-full max-w-md rounded-xl border border-border bg-card p-6">
            <button className="absolute right-4 top-4 text-muted-foreground" onClick={() => setIsImportOpen(false)}>
              <X className="h-4 w-4" />
            </button>
            <h2 className="mb-2 text-lg font-semibold">Import CSV</h2>
            <p className="mb-4 text-xs text-muted-foreground">
              Columns: phone_number, first_name, last_name, email, tags.
            </p>
            <form onSubmit={handleImportContacts} className="space-y-4">
              <Input type="file" accept=".csv" onChange={(e) => setCsvFile(e.target.files?.[0] || null)} />
              <div className="flex justify-end gap-2">
                <Button type="button" variant="outline" onClick={() => setIsImportOpen(false)}>
                  Cancel
                </Button>
                <Button type="submit" disabled={importing}>
                  Import
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
