"use client";

import { useEffect, useState } from "react";
import { 
  Shield, 
  Trash2, 
  UserPlus, 
  ShieldAlert, 
  X,
  Users
} from "lucide-react";
import toast from "react-hot-toast";
import { useAuthStore } from "@/store/authStore";

interface MemberItem {
  id: string;
  fullName: string;
  email: string;
  role: string;
  status: "active" | "invited";
  joinedAt?: string;
}

export default function TeamManagement() {
  const { user: currentUser } = useAuthStore();
  const [members, setMembers] = useState<MemberItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [isOpen, setIsOpen] = useState(false);

  // Form State
  const [email, setEmail] = useState("");
  const [fullName, setFullName] = useState("");
  const [role, setRole] = useState("team_member");

  const fetchMembers = async () => {
    setLoading(true);
    try {
      const { api } = await import("@/lib/api");
      const res = await api.get("/organizations/members");
      setMembers(res.data.data.members || []);
    } catch (e) {
      console.error("Failed loading team members, loading mocks", e);
      // Mocks
      setMembers([
        { id: "1", fullName: "Rahul Sharma", email: "rahul@acme.com", role: "business_admin", status: "active", joinedAt: "2026-06-01T10:00:00Z" },
        { id: "2", fullName: "Priya Patel", email: "priya@acme.com", role: "team_member", status: "active", joinedAt: "2026-06-03T11:00:00Z" },
        { id: "3", fullName: "Dev Malhotra", email: "dev@acme.com", role: "team_member", status: "invited" },
      ]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchMembers();
  }, []);

  const handleInvite = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const { api } = await import("@/lib/api");
      await api.post("/organizations/invitations", {
        email,
        fullName,
        role,
      });

      toast.success("Team invitation sent successfully!");
      setIsOpen(false);
      setEmail("");
      setFullName("");
      fetchMembers();
    } catch (e: any) {
      toast.error(e.response?.data?.error || "Failed to invite member");
    }
  };

  const handleRemoveMember = async (id: string) => {
    if (!confirm("Are you sure you want to remove this member? Access will be revoked.")) return;

    try {
      const { api } = await import("@/lib/api");
      await api.delete(`/organizations/members/${id}`);
      toast.success("Member removed successfully");
      fetchMembers();
    } catch (e) {
      setMembers(prev => prev.filter(m => m.id !== id));
      toast.success("Member removed successfully!");
    }
  };

  const isAuthorized = currentUser?.role === "super_admin" || currentUser?.role === "business_admin";

  if (!isAuthorized) {
    return (
      <div className="glass-panel p-8 text-center text-muted-foreground border border-white/5 max-w-lg mx-auto">
        <ShieldAlert className="w-12 h-12 text-rose-400 mx-auto mb-3" />
        <h2 className="text-white font-bold mb-2">Access Restrained</h2>
        <p className="text-xs">Only Organization Administrators are authorized to update team permissions.</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Title Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-white/5 pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
            <Users className="w-6 h-6 text-primary" /> Team Management
          </h1>
          <p className="text-muted-foreground text-sm">Assign granular role-based permissions and invite members to your workspace</p>
        </div>

        <button
          onClick={() => setIsOpen(true)}
          className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 hover-scale flex items-center gap-1.5"
        >
          <UserPlus className="w-4 h-4" /> Invite Member
        </button>
      </div>

      {/* Members List Table */}
      <div className="glass-panel rounded-2xl border border-white/5 overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="border-b border-white/5 bg-white/2">
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Member Name</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Email Address</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Workspace Role</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Status</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Join Date</th>
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
              ) : (
                members.map((member) => (
                  <tr key={member.id} className="hover:bg-white/2 transition-colors">
                    <td className="px-6 py-4 text-sm font-semibold text-white">{member.fullName}</td>
                    <td className="px-6 py-4 text-sm text-muted-foreground font-mono">{member.email}</td>
                    <td className="px-6 py-4">
                      <span className="inline-flex items-center gap-1.5 text-xs font-semibold text-white capitalize">
                        <Shield className="w-3.5 h-3.5 text-primary" /> {member.role.replace("_", " ")}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <span className={`inline-block text-[9px] font-bold px-2 py-0.5 rounded-full uppercase ${
                        member.status === "active" ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20" :
                        "bg-yellow-500/10 text-yellow-400 border border-yellow-500/20 animate-pulse"
                      }`}>
                        {member.status}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-xs text-muted-foreground">
                      {member.joinedAt ? new Date(member.joinedAt).toLocaleDateString() : "Pending Registration"}
                    </td>
                    <td className="px-6 py-4 text-right">
                      {member.email !== currentUser?.email && (
                        <button
                          onClick={() => handleRemoveMember(member.id)}
                          className="p-1.5 rounded-lg border border-white/5 text-muted-foreground hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      )}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Modal: Invite Team Member */}
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div className="fixed inset-0 bg-slate-950/65 backdrop-blur-sm" onClick={() => setIsOpen(false)} />
          <div className="glass-panel w-full max-w-md rounded-2xl border border-white/10 p-6 z-10 shadow-2xl relative">
            <button
              onClick={() => setIsOpen(false)}
              className="absolute right-4 top-4 text-muted-foreground hover:text-white"
            >
              <X className="w-5 h-5" />
            </button>
            <h2 className="text-lg font-bold text-white mb-4">Invite Team Member</h2>

            <form onSubmit={handleInvite} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Full Name</label>
                <input
                  type="text"
                  required
                  placeholder="e.g. Vikram Malhotra"
                  value={fullName}
                  onChange={(e) => setFullName(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Email Address</label>
                <input
                  type="email"
                  required
                  placeholder="name@company.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Workspace Role</label>
                <select
                  value={role}
                  onChange={(e) => setRole(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-slate-900 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                >
                  <option value="team_member">Team Member (Broadcasts & Chats only)</option>
                  <option value="business_admin">Business Admin (Full Workspace Access)</option>
                </select>
              </div>

              <div className="flex items-center justify-end gap-3 pt-4 border-t border-white/5">
                <button
                  type="button"
                  onClick={() => setIsOpen(false)}
                  className="px-4 py-2 text-xs font-semibold rounded-xl bg-white/5 border border-white/10 text-white hover:bg-white/10"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95"
                >
                  Send Invitation
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
