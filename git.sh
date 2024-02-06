COMMIT=$(</dev/stdin)

echo $COMMIT

git add .
git commit -m "$COMMIT"
git push