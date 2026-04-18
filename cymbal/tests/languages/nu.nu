def greet [name: string] {
  echo $"Hello, ($name)!"
}

def add [a: int, b: int] -> int {
  $a + $b
}

def "str repeat" [s: string, n: int] -> string {
  1..$n | each { $s } | str join
}

def fibonacci [n: int] -> int {
  if $n <= 1 { $n } else {
    (fibonacci ($n - 1)) + (fibonacci ($n - 2))
  }
}
