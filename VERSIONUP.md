# How to Version Up

1. Make sure the changes are merged into master branch.
1. Create `version\_up` branch and checkout there.
1. Regenerate the documentation.
    1. `git rm -rf docs`
    1. `cargo doc`
    1. `mv target/doc docs`
    1. `git add docs`
    1. `git commit -m 'Regenerate documents'`
1. Update CHANGELOG.md and and `package.version` in Cargo.toml, and commit.
1. Merge branch `version\_up` into master.
1. Create a new version tag.
1. Reset 'docs' branch to 'master'
1. Push docs branch to origin.
1. Check "https://wbcchsyn.github.io/rust-mouse-sovec/mouse\_sovec/"
1. Push branch master, and the new tag to origin.
