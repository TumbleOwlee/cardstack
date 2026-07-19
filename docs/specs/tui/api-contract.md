# TUI — API Contract

The stable operator-facing surface: the exhaustive `:` command list and every
keybinding, by context. These names, syntax, and key mappings are the contract
and shall not change without a spec change.

## `:` commands

| Command | Syntax | Effect |
|---|---|---|
| New task | `:new-task` | Opens the task detail dialog blank (`UI-R-052`) |
| New board | `:new-board <name>` | Creates and switches to a new empty board (`UI-R-053`) |
| Rename board | `:rename <name>` | Renames the active board (`UI-R-057`) |
| Swap tabs | `:swap <i> <j>` | Swaps the tab positions of boards `<i>` and `<j>` (`UI-R-058`) |
| Delete board | `:delete` | Opens a confirm dialog, then deletes the active board (`UI-R-059`) |
| Manage categories | `:categories` | Opens the category-management dialog (`UI-R-043`) |
| Quit | `:q` | Exits the application |

## Keybindings — board view (no dialog open)

| Key | Effect |
|---|---|
| `h` / `←` | Focus previous column (`UI-R-021`) |
| `l` / `→` | Focus next column (`UI-R-021`) |
| `j` / `↓` | Focus next card in column (`UI-R-022`) |
| `k` / `↑` | Focus previous card in column (`UI-R-022`) |
| `H` / `shift+←` | Move focused card's status one column left (`UI-R-030`) |
| `L` / `shift+→` | Move focused card's status one column right (`UI-R-030`) |
| `J` / `shift+↓` | Reorder focused card down within its column (`UI-R-031`) |
| `K` / `shift+↑` | Reorder focused card up within its column (`UI-R-031`) |
| `Enter` | Open task detail dialog on focused card (`UI-R-040`) |
| `d` | Open delete-confirmation dialog on focused card (`UI-R-042`) |
| `:` | Enter command-line mode (`UI-R-050`) |
| `q` | Quit |

## Keybindings — board tab bar

| Key | Effect |
|---|---|
| `[` / `]` (or scroll) | Switch to previous/next board tab |
| `Ctrl+T` then a digit | Switch directly to the board at that index (`UI-R-004`) |

## Keybindings — task detail / category-management / confirm dialogs

| Key | Effect |
|---|---|
| `Tab` / `shift+Tab` | Move between fields |
| `Enter` (on confirm control) | Confirm and apply (`UI-R-040`, `UI-R-042`, `UI-R-043`) |
| `Esc` | Close without applying (`UI-R-044`) |
