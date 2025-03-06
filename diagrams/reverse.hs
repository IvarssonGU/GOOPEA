data List = Nil | Cons List Int
    
reverseHelper :: List -> List -> List
reverseHelper list acc = case list of
    Cons xs x -> reverseHelper xs (Cons acc x)
    Nil -> acc
    
reverseList :: List -> List
reverseList xs = reverseHelper xs Nil