set -e
zola build
# Forgetting to exclude `.git` has costed me several days worth of work (and over 30 commits).
find . -maxdepth 1 -not -name '.' -not -name '.git' -not -name 'public' -exec rm -r {} \;
mv public/* .
rmdir public/
git add .
git commit --amend -m "Deploy site"
git push --force
git checkout master
