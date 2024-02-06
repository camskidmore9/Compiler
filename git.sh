COMMIT=$1

echo $COMMIT

git add .
git commit -m "$COMMIT"
git push