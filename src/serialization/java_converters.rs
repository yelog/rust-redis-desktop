/// Check if a class name is a known collection type
pub fn is_collection_type(class_name: &str) -> bool {
    matches!(
        class_name,
        "java.util.ArrayList"
            | "java.util.LinkedList"
            | "java.util.Vector"
            | "java.util.Stack"
            | "java.util.CopyOnWriteArrayList"
    )
}

/// Check if a class name is a known map type
pub fn is_map_type(class_name: &str) -> bool {
    matches!(
        class_name,
        "java.util.HashMap"
            | "java.util.LinkedHashMap"
            | "java.util.TreeMap"
            | "java.util.Hashtable"
            | "java.util.IdentityHashMap"
            | "java.util.WeakHashMap"
            | "java.util.concurrent.ConcurrentHashMap"
    )
}

/// Check if a class name is a known set type
pub fn is_set_type(class_name: &str) -> bool {
    matches!(
        class_name,
        "java.util.HashSet"
            | "java.util.LinkedHashSet"
            | "java.util.TreeSet"
            | "java.util.CopyOnWriteArraySet"
    )
}

/// Get a friendly display name for a Java class
pub fn get_collection_display_name(class_name: &str) -> &'static str {
    match class_name {
        "java.util.ArrayList" => "ArrayList",
        "java.util.LinkedList" => "LinkedList",
        "java.util.Vector" => "Vector",
        "java.util.Stack" => "Stack",
        "java.util.HashMap" => "HashMap",
        "java.util.LinkedHashMap" => "LinkedHashMap",
        "java.util.TreeMap" => "TreeMap",
        "java.util.Hashtable" => "Hashtable",
        "java.util.HashSet" => "HashSet",
        "java.util.LinkedHashSet" => "LinkedHashSet",
        "java.util.TreeSet" => "TreeSet",
        "java.util.concurrent.ConcurrentHashMap" => "ConcurrentHashMap",
        _ => "Collection",
    }
}
