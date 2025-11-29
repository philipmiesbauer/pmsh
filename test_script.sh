echo "Hello from script"
pwd
VAR1="outside"
echo $VAR1
(
    cd /tmp
    pwd
    echo "Pipeline test" | wc -w
    VAR1="inside"
    echo $VAR1
)
pwd
echo $VAR1
VAR1="outside"
echo $VAR1
