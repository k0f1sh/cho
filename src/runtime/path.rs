use crate::ast::PathPart;

pub(super) fn part(value: &str, part: PathPart) -> String {
    let path = trim_trailing_separators(value);
    match part {
        PathPart::Name => name(path).to_owned(),
        PathPart::Stem => stem(name(path)).to_owned(),
        PathPart::Extension => extension(name(path)).to_owned(),
        PathPart::Directory => directory(path).to_owned(),
    }
}

pub(super) fn function_name(part: &PathPart) -> &'static str {
    match part {
        PathPart::Name => "path/name",
        PathPart::Stem => "path/stem",
        PathPart::Extension => "path/ext",
        PathPart::Directory => "path/dir",
    }
}

fn trim_trailing_separators(value: &str) -> &str {
    let trimmed = value.trim_end_matches('/');
    if trimmed.is_empty() && value.starts_with('/') {
        "/"
    } else {
        trimmed
    }
}

fn name(path: &str) -> &str {
    if path == "/" {
        ""
    } else {
        path.rsplit('/').next().unwrap_or("")
    }
}

fn extension(name: &str) -> &str {
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return "";
    };
    if stem.is_empty() || extension.is_empty() {
        ""
    } else {
        extension
    }
}

fn stem(name: &str) -> &str {
    let extension = extension(name);
    if extension.is_empty() {
        name
    } else {
        &name[..name.len() - extension.len() - 1]
    }
}

fn directory(path: &str) -> &str {
    if path == "/" {
        return "/";
    }
    let Some((directory, _)) = path.rsplit_once('/') else {
        return "";
    };
    let directory = directory.trim_end_matches('/');
    if directory.is_empty() && path.starts_with('/') {
        "/"
    } else {
        directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_unix_path_parts_lexically() {
        assert_eq!(part("/var/log/app.log", PathPart::Name), "app.log");
        assert_eq!(part("archive.tar.gz", PathPart::Stem), "archive.tar");
        assert_eq!(part("archive.tar.gz", PathPart::Extension), "gz");
        assert_eq!(part("/var/log/app.log", PathPart::Directory), "/var/log");
    }
}
