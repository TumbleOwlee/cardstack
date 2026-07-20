use crate::model::Task;

/// UI-R-060 — how a `label=<expr>` term combines its listed labels.
#[derive(Debug, Clone, PartialEq, Eq)]
enum LabelExpr {
    /// `&`-separated: the task must carry every listed label.
    All(Vec<String>),
    /// `|`-separated (or a single label): the task must carry any listed label.
    Any(Vec<String>),
}

/// UI-R-060 — an active `:filter` condition, scoped to one board and held only
/// in memory. Values are stored lowercased for case-insensitive matching
/// (mirrors `Board::label_colors`' keying); `raw` keeps the operator's original
/// condition text for the status line (`UI-R-061`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    raw: String,
    /// `|`-separated category names (OR), lowercased. A task has at most one
    /// category, so `&` is not allowed here.
    category: Option<Vec<String>>,
    labels: Option<LabelExpr>,
}

impl Filter {
    /// UI-R-060 — parse the text typed after `:filter` (without the leading
    /// `:filter`). `Ok(None)` means "clear the filter" (empty input or the
    /// literal `clear`). `Err(())` is a syntactically invalid condition, which
    /// the caller surfaces as a command-line error (`UI-R-051`).
    pub fn parse(input: &str) -> Result<Option<Filter>, ()> {
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed == "clear" {
            return Ok(None);
        }

        let mut category: Option<Vec<String>> = None;
        let mut labels: Option<LabelExpr> = None;

        for term in trimmed.split_whitespace() {
            let (key, value) = term.split_once('=').ok_or(())?;
            if value.is_empty() {
                return Err(());
            }
            match key {
                "category" => {
                    if category.is_some() {
                        return Err(());
                    }
                    // A task has at most one category, so `&` is meaningless
                    // here; only `|` (OR) or a single name is valid.
                    if value.contains('&') {
                        return Err(());
                    }
                    let names: Vec<String> = value.split('|').map(|p| p.to_lowercase()).collect();
                    if names.iter().any(|p| p.is_empty()) {
                        return Err(());
                    }
                    category = Some(names);
                }
                "label" => {
                    if labels.is_some() {
                        return Err(());
                    }
                    labels = Some(parse_label_expr(value)?);
                }
                _ => return Err(()),
            }
        }

        Ok(Some(Filter {
            raw: trimmed.to_string(),
            category,
            labels,
        }))
    }

    /// UI-R-060 — a task matches when it satisfies every present term (category
    /// AND label). Comparisons are case-insensitive.
    pub fn matches(&self, task: &Task) -> bool {
        if let Some(cats) = &self.category {
            match &task.category {
                Some(tc) if cats.contains(&tc.to_lowercase()) => {}
                _ => return false,
            }
        }
        if let Some(expr) = &self.labels {
            let has = |want: &str| task.labels.iter().any(|l| l.to_lowercase() == want);
            let ok = match expr {
                LabelExpr::All(ls) => ls.iter().all(|l| has(l)),
                LabelExpr::Any(ls) => ls.iter().any(|l| has(l)),
            };
            if !ok {
                return false;
            }
        }
        true
    }

    /// UI-R-061 — the condition text shown in the filter-status line.
    pub fn describe(&self) -> &str {
        &self.raw
    }
}

/// UI-R-060 — parse a `label=` value: a single label, an `&`-joined list
/// (`All`), or a `|`-joined list (`Any`). Mixing `&` and `|` is an error.
fn parse_label_expr(value: &str) -> Result<LabelExpr, ()> {
    let has_and = value.contains('&');
    let has_or = value.contains('|');
    if has_and && has_or {
        return Err(());
    }
    let split = |sep: char| -> Result<Vec<String>, ()> {
        let parts: Vec<String> = value.split(sep).map(|p| p.to_lowercase()).collect();
        if parts.iter().any(|p| p.is_empty()) {
            return Err(());
        }
        Ok(parts)
    };
    if has_and {
        Ok(LabelExpr::All(split('&')?))
    } else {
        // A single label (no separator) parses to a one-element `Any`.
        Ok(LabelExpr::Any(split('|')?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Status;

    fn task(category: Option<&str>, labels: &[&str]) -> Task {
        Task {
            id: 0,
            title: "t".to_string(),
            description: String::new(),
            due_date: None,
            category: category.map(|c| c.to_string()),
            labels: labels.iter().map(|l| l.to_string()).collect(),
            status: Status::Open,
        }
    }

    /// UI-R-060 — empty input and `clear` both mean "clear the filter".
    #[test]
    fn ut_parse_clear_forms() {
        assert_eq!(Filter::parse(""), Ok(None));
        assert_eq!(Filter::parse("   "), Ok(None));
        assert_eq!(Filter::parse("clear"), Ok(None));
    }

    /// UI-R-060 — a lone `category=` term matches on category, case-insensitively.
    #[test]
    fn ut_parse_and_match_category() {
        let f = Filter::parse("category=Work").unwrap().unwrap();
        assert!(f.matches(&task(Some("work"), &[])));
        assert!(f.matches(&task(Some("WORK"), &[])));
        assert!(!f.matches(&task(Some("home"), &[])));
        assert!(!f.matches(&task(None, &[])));
    }

    /// UI-R-060 — a `|` category list matches any of the listed categories.
    #[test]
    fn ut_match_category_or() {
        let f = Filter::parse("category=support|enbas").unwrap().unwrap();
        assert!(f.matches(&task(Some("support"), &[])));
        assert!(f.matches(&task(Some("ENBAS"), &[])));
        assert!(!f.matches(&task(Some("other"), &[])));
        assert!(!f.matches(&task(None, &[])));
    }

    /// UI-R-060 — a single-label term matches any task carrying that label.
    #[test]
    fn ut_parse_and_match_single_label() {
        let f = Filter::parse("label=Bug").unwrap().unwrap();
        assert!(f.matches(&task(None, &["bug"])));
        assert!(f.matches(&task(None, &["BUG", "other"])));
        assert!(!f.matches(&task(None, &["feature"])));
    }

    /// UI-R-060 — an `&` list requires all listed labels.
    #[test]
    fn ut_match_label_and() {
        let f = Filter::parse("label=bug&urgent").unwrap().unwrap();
        assert!(f.matches(&task(None, &["bug", "urgent"])));
        assert!(!f.matches(&task(None, &["bug"])));
    }

    /// UI-R-060 — a `|` list requires any listed label.
    #[test]
    fn ut_match_label_or() {
        let f = Filter::parse("label=bug|urgent").unwrap().unwrap();
        assert!(f.matches(&task(None, &["urgent"])));
        assert!(f.matches(&task(None, &["bug"])));
        assert!(!f.matches(&task(None, &["feature"])));
    }

    /// UI-R-060 — both terms present are ANDed together.
    #[test]
    fn ut_match_category_and_label() {
        let f = Filter::parse("category=work label=bug").unwrap().unwrap();
        assert!(f.matches(&task(Some("work"), &["bug"])));
        assert!(!f.matches(&task(Some("work"), &["feature"])));
        assert!(!f.matches(&task(Some("home"), &["bug"])));
    }

    /// UI-R-060 — mixed `&`/`|`, unknown key, empty value, and duplicate key
    /// are all rejected.
    #[test]
    fn ut_parse_rejects_invalid() {
        assert_eq!(Filter::parse("label=a&b|c"), Err(()));
        assert_eq!(Filter::parse("category=a&b"), Err(()));
        assert_eq!(Filter::parse("category=a|"), Err(()));
        assert_eq!(Filter::parse("foo=bar"), Err(()));
        assert_eq!(Filter::parse("category="), Err(()));
        assert_eq!(Filter::parse("label=a&"), Err(()));
        assert_eq!(Filter::parse("category=a category=b"), Err(()));
        assert_eq!(Filter::parse("bareword"), Err(()));
    }

    /// UI-R-061 — `describe` returns the original condition text.
    #[test]
    fn ut_describe_is_raw_text() {
        let f = Filter::parse("category=work label=bug").unwrap().unwrap();
        assert_eq!(f.describe(), "category=work label=bug");
    }
}
