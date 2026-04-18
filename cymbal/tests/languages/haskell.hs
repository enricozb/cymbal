module Main where

import Data.List (sort)
import Data.Maybe (fromMaybe)

data Shape
  = Circle Double
  | Rectangle Double Double
  | Triangle Double Double Double

data Color = Red | Green | Blue

type Name = String

type Palette = [Color]

area :: Shape -> Double
area (Circle r) = pi * r * r
area (Rectangle w h) = w * h
area (Triangle a b c) =
  let s = (a + b + c) / 2
   in sqrt (s * (s - a) * (s - b) * (s - c))

perimeter :: Shape -> Double
perimeter (Circle r) = 2 * pi * r
perimeter (Rectangle w h) = 2 * (w + h)
perimeter (Triangle a b c) = a + b + c

greet :: Name -> String
greet name = "Hello, " ++ name ++ "!"

safeHead :: [a] -> Maybe a
safeHead [] = Nothing
safeHead (x : _) = Just x
