MAX_RETRIES = 3


class Animal:
    def __init__(self, name: str) -> None:
        self.name = name

    def speak(self) -> str:
        raise NotImplementedError

    @staticmethod
    def kingdom() -> str:
        return "Animalia"


class Dog(Animal):
    def speak(self) -> str:
        return f"{self.name} says: woof!"

    def fetch(self, item: str) -> str:
        return f"{self.name} fetched {item}"


def greet(name: str) -> str:
    return f"Hello, {name}!"


def fibonacci(n: int) -> int:
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)
