type color = Red | Green | Blue

type point = { x : float; y : float }

type 'a result =
  | Ok of 'a
  | Err of string

let pi = 3.14159265358979

let distance p1 p2 =
  let dx = p1.x -. p2.x in
  let dy = p1.y -. p2.y in
  sqrt ((dx *. dx) +. (dy *. dy))

let rec factorial n =
  if n <= 0 then 1 else n * factorial (n - 1)

let greet name = "Hello, " ^ name ^ "!"

let map_result f = function
  | Ok v -> Ok (f v)
  | Err e -> Err e

module StringMap = Map.Make (String)

module type COLLECTION = sig
  type 'a t

  val empty : 'a t
  val insert : 'a -> 'a t -> 'a t
  val member : 'a -> 'a t -> bool
end
