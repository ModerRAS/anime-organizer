$content = Get-Content 'D:\WorkSpace\Rust\anime-organizer\src\parser.rs' -Raw

# Pattern 1: Fix the common pattern
# Change: 
#     let result = FilenameParser::parse(&path);
#     if let Some(info) = result {
# To:
#     let result = FilenameParser::parse(&path).expect("Parser should handle this format");

$pattern1 = 'let result = FilenameParser::parse\(&path\);\s*if let Some\(info\) = result \{'
$replacement1 = 'let info = FilenameParser::parse(&path).expect("Parser should handle this format");'
$content = $content -replace $pattern1, $replacement1

Set-Content -Path 'D:\WorkSpace\Rust\anime-organizer\src\parser.rs' -Value $content -NoNewline
Write-Host 'Phase 1 complete: Basic pattern replacement'
