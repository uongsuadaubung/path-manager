// Centralized English language definitions for path-mgr

#[allow(dead_code)]
pub mod text {
    // Header & App Info
    pub const APP_TITLE: &str = " PATH MANAGER ";
    pub const SUBTITLE: &str = "Windows Environment PATH Editor";
    pub const TAB_USER: &str = "[1] User PATH";
    pub const TAB_SYSTEM: &str = "[2] System PATH";
    pub const SCOPE_USER: &str = "User";
    pub const SCOPE_SYSTEM: &str = "System";

    // Table & Columns
    pub const COL_INDEX: &str = "#";
    pub const COL_STATUS: &str = "Status";
    pub const COL_PATH: &str = "Path";

    // Status Tags
    pub const STATUS_OK: &str = "OK";
    pub const STATUS_BROKEN: &str = "BROKEN";
    pub const STATUS_NO_EXEC: &str = "NO-EXEC";
    pub const STATUS_SHADOWED: &str = "SHADOWED";

    // Details Panel
    pub const DETAILS_TITLE: &str = " Path Details & Diagnostics ";
    pub const DETAILS_FULL_PATH: &str = "Target Path";
    pub const DETAILS_EXPANDED_PATH: &str = "Expanded Path";
    pub const DETAILS_STATUS_EXISTS: &str = "Directory Exists";
    pub const DETAILS_STATUS_MISSING: &str = "Directory Not Found (Broken Path)";
    pub const DETAILS_HAS_EXEC: &str = "Executables (.exe, .bat, .cmd, ...) Found";
    pub const DETAILS_NO_EXEC: &str = "Warning: No Executable Files Found in Directory";
    pub const DETAILS_SHADOWED_DESC: &str = "Note: Already present in System PATH (redundant in User PATH)";
    pub const DETAILS_REG_KEY: &str = "Registry Key";
    pub const DETAILS_NO_SELECTION: &str = "No path selected.";

    // Status Messages
    pub const STATUS_READY: &str = "Ready.";
    pub const STATUS_ADDED: &str = "Successfully added path: ";
    pub const STATUS_UPDATED: &str = "Successfully updated path.";
    pub const STATUS_DELETED: &str = "Successfully deleted path.";
    pub const STATUS_REORDERED: &str = "Path order updated.";
    pub const STATUS_REFRESHED: &str = "Paths refreshed from Registry.";
    pub const STATUS_REGISTRY_CHANGED: &str = "Registry change detected, updated automatically.";
    pub const STATUS_COPIED: &str = "Path copied to clipboard.";
    pub const STATUS_ERROR_PREFIX: &str = "Error: ";
    pub const STATUS_NO_DUPLICATES: &str = "No duplicates found.";

    // Modals
    pub const MODAL_ADD_TITLE: &str = " Add New Path ";
    pub const MODAL_ADD_PROMPT: &str = "Enter folder path (e.g. C:\\Tools\\bin):";
    pub const MODAL_EDIT_TITLE: &str = " Edit Path ";
    pub const MODAL_EDIT_PROMPT: &str = "Modify folder path:";
    pub const MODAL_DELETE_TITLE: &str = " Confirm Delete ";
    pub const MODAL_DELETE_PROMPT: &str = "Are you sure you want to remove this path from PATH?";
    pub const MODAL_CONFIRM_HINT: &str = "Press [y/Enter] to Confirm, [n/Esc] to Cancel";
    pub const MODAL_HELP_TITLE: &str = " Keyboard Shortcuts & Help ";

    // Footer Keybindings
    pub const FOOTER_NAV: &str = "[↑/↓/k/j] Navigate";
    pub const FOOTER_TAB: &str = "[1/2/Tab] Scope";
    pub const FOOTER_ADD: &str = "[a] Add";
    pub const FOOTER_EDIT: &str = "[e] Edit";
    pub const FOOTER_DELETE: &str = "[d/Del] Delete";
    pub const FOOTER_MOVE: &str = "[K/J] Move Up/Down";
    pub const FOOTER_SEARCH: &str = "[/] Search";
    pub const FOOTER_DEDUPE: &str = "[c] Clean Duplicates";
    pub const FOOTER_BACKUP: &str = "[b] Backup";
    pub const FOOTER_RESTORE: &str = "[i] Import";
    pub const FOOTER_REFRESH: &str = "[r] Refresh";
    pub const FOOTER_HELP: &str = "[?] Help";
    pub const FOOTER_QUIT: &str = "[q] Quit";

    // CLI Text
    pub const CLI_CURRENT_PATHS: &str = "Current PATH Entries";
    pub const CLI_TOTAL_PATHS: &str = "Total paths: ";
    pub const CLI_PATH_EXISTS: &str = "Path already exists in PATH.";
    pub const CLI_UPDATED_SUCCESS: &str = "Successfully updated PATH entry:";
    pub const CLI_FROM: &str = "  Old: ";
    pub const CLI_TO: &str = "  New: ";
}
