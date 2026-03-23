puts $1
     ^^ Style/PerlBackrefs: Prefer `Regexp.last_match(1)` over `$1`.

$9
^^ Style/PerlBackrefs: Prefer `Regexp.last_match(9)` over `$9`.

$&
^^ Style/PerlBackrefs: Prefer `Regexp.last_match(0)` over `$&`.

$MATCH
^^^^^^ Style/PerlBackrefs: Prefer `Regexp.last_match(0)` over `$MATCH`.

$PREMATCH
^^^^^^^^^ Style/PerlBackrefs: Prefer `Regexp.last_match.pre_match` over `$PREMATCH`.

$POSTMATCH
^^^^^^^^^^ Style/PerlBackrefs: Prefer `Regexp.last_match.post_match` over `$POSTMATCH`.

$LAST_PAREN_MATCH
^^^^^^^^^^^^^^^^^ Style/PerlBackrefs: Prefer `Regexp.last_match(-1)` over `$LAST_PAREN_MATCH`.
