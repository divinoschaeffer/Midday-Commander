use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

#[derive(Debug, PartialEq, Clone)]
pub enum FsNodeType {
    File,
    Directory,
}

#[derive(Debug)]
pub struct FsNode {
    pub name: String,
    pub path: PathBuf,
    pub node_type: FsNodeType,
    pub parent: Option<Weak<RefCell<FsNode>>>,
    pub children: Vec<Rc<RefCell<FsNode>>>,
}

impl FsNode {
    pub fn new(
        name: String,
        path: PathBuf,
        fs_node_type: FsNodeType,
        parent: Option<Weak<RefCell<FsNode>>>,
        children: Vec<Rc<RefCell<FsNode>>>
    ) -> FsNode {
        FsNode {
            name,
            path,
            node_type: fs_node_type,
            parent,
            children,
        }
    }

    /// add child to a node
    pub fn add_child(parent: &Rc<RefCell<FsNode>>, child:FsNode) {
        let child = Rc::new(RefCell::new(child));
        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        parent.borrow_mut().children.push(child);
    }

    /// find node amongst the direct children of a node
    pub fn find_node(
        &mut self,
        path: PathBuf,
        fs_node_type: Option<FsNodeType>
    ) -> Option<Rc<RefCell<FsNode>>> {
        self.children.iter().find_map(|child| {
            let node = child.borrow();
            if node.path == path
                && (fs_node_type.is_none() || (fs_node_type.as_ref() == Some(&node.node_type))) {
                Some(Rc::clone(child))
            }
            else {
                None
            }
        })
    }

    /// remove node if there is one and return it
    pub fn remove_node(&mut self, path: PathBuf, fs_node_type: Option<FsNodeType>) -> Option<Rc<RefCell<FsNode>>> {
        let position = self.children
            .iter()
            .position(|child|
                child.borrow().path == path
                    && (fs_node_type.is_none() || (fs_node_type.as_ref() == Some(&child.borrow().node_type)))
            );

        if let Some(position) = position {
            Some(Rc::clone(&self.children.remove(position)))
        } else {
            None
        }
    }

    pub fn create_node_from_path(path: PathBuf, parent: Option<Weak<RefCell<FsNode>>>) -> Option<Rc<RefCell<FsNode>>> {
        let name = match path.file_name() {
            Some(file_name) => match file_name.to_str() {
                Some(name_str) => name_str.to_string(),
                None => return None,
            },
            None => return None,
        };

        let node_type = if path.is_file() {
            FsNodeType::File
        } else {
            FsNodeType::Directory
        };

        // Create the current node
        let node = FsNode::new(name, path.clone(), node_type.clone(), parent, vec![]);
        let node_rc = Rc::new(RefCell::new(node));

        // If it's a directory, recursively process all its children
        if node_type == FsNodeType::Directory {
            match std::fs::read_dir(&path) {
                Ok(entries) => {
                    // Process each entry in the directory
                    for entry_result in entries {
                        if let Ok(entry) = entry_result {
                            let child_path = entry.path();

                            // Recursively create child node with this node as parent
                            if let Some(child_node) = Self::create_node_from_path(
                                child_path,
                                Some(Rc::downgrade(&node_rc))
                            ) {
                                // Add child to parent's children list
                                node_rc.borrow_mut().children.push(child_node);
                            }
                        }
                    }
                },
                Err(_) => {
                    // Handle directory read error - could return None or keep the node without children
                    // Here we choose to keep the node without children
                }
            }
        }

        Some(node_rc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;
    use std::io::Write;

    // Helper function to create a test directory structure
    fn create_test_directory() -> TempDir {
        let temp_dir = TempDir::new("fsnode_tests").expect("Failed to create temp dir");

        // Create a simple directory structure:
        // temp_dir/
        // ├── file1.txt
        // ├── file2.txt
        // └── subdir/
        //     ├── file3.txt
        //     └── nested/
        //         └── file4.txt

        // Create files in root directory
        let file1_path = temp_dir.path().join("file1.txt");
        let mut file1 = fs::File::create(&file1_path).expect("Failed to create file1.txt");
        file1.write_all(b"Content of file1").expect("Failed to write to file1.txt");

        let file2_path = temp_dir.path().join("file2.txt");
        let mut file2 = fs::File::create(&file2_path).expect("Failed to create file2.txt");
        file2.write_all(b"Content of file2").expect("Failed to write to file2.txt");

        // Create subdirectory
        let subdir_path = temp_dir.path().join("subdir");
        fs::create_dir(&subdir_path).expect("Failed to create subdir");

        // Create file in subdirectory
        let file3_path = subdir_path.join("file3.txt");
        let mut file3 = fs::File::create(&file3_path).expect("Failed to create file3.txt");
        file3.write_all(b"Content of file3").expect("Failed to write to file3.txt");

        // Create nested subdirectory
        let nested_path = subdir_path.join("nested");
        fs::create_dir(&nested_path).expect("Failed to create nested directory");

        // Create file in nested subdirectory
        let file4_path = nested_path.join("file4.txt");
        let mut file4 = fs::File::create(&file4_path).expect("Failed to create file4.txt");
        file4.write_all(b"Content of file4").expect("Failed to write to file4.txt");

        temp_dir
    }

    #[test]
    fn test_create_node_from_path() {
        let temp_dir = create_test_directory();

        // Create a node from the root directory
        let root_node = FsNode::create_node_from_path(temp_dir.path().to_path_buf(), None)
            .expect("Failed to create root node");

        // Check root node properties
        let root = root_node.borrow();
        assert_eq!(root.node_type, FsNodeType::Directory);
        assert_eq!(root.path, temp_dir.path().to_path_buf());
        // assert_eq!(root.parent, None);

        // Root should have 3 children (file1.txt, file2.txt, subdir)
        assert_eq!(root.children.len(), 3);

        // Find subdir node
        let subdir_path = temp_dir.path().join("subdir");
        let mut children_names: Vec<String> = root.children.iter()
            .map(|child| child.borrow().name.clone())
            .collect();
        children_names.sort();

        assert!(children_names.contains(&"file1.txt".to_string()));
        assert!(children_names.contains(&"file2.txt".to_string()));
        assert!(children_names.contains(&"subdir".to_string()));

        // Find the subdir node
        let subdir_node = root.children.iter()
            .find(|child| child.borrow().name == "subdir")
            .expect("Subdir node not found");

        // Check subdir properties
        let subdir = subdir_node.borrow();
        assert_eq!(subdir.node_type, FsNodeType::Directory);
        assert_eq!(subdir.path, subdir_path);
        assert!(subdir.parent.is_some());

        // Subdir should have 2 children (file3.txt, nested)
        assert_eq!(subdir.children.len(), 2);

        // Check that the subdir's parent is the root node
        let subdir_parent = subdir.parent.as_ref()
            .and_then(|weak| weak.upgrade())
            .expect("Failed to get parent of subdir");

        assert_eq!(subdir_parent.borrow().path, root.path);
    }

    #[test]
    fn test_find_node() {
        let temp_dir = create_test_directory();

        // Create a node tree from the root directory
        let root_node = FsNode::create_node_from_path(temp_dir.path().to_path_buf(), None)
            .expect("Failed to create root node");

        let file1_path = temp_dir.path().join("file1.txt");
        let subdir_path = temp_dir.path().join("subdir");

        // Find file1.txt
        let mut root_mut = root_node.borrow_mut();
        let file1_node = root_mut.find_node(file1_path.clone(), Some(FsNodeType::File))
            .expect("Failed to find file1.txt");

        assert_eq!(file1_node.borrow().name, "file1.txt");
        assert_eq!(file1_node.borrow().path, file1_path);
        assert_eq!(file1_node.borrow().node_type, FsNodeType::File);

        // Find subdir
        let subdir_node = root_mut.find_node(subdir_path.clone(), Some(FsNodeType::Directory))
            .expect("Failed to find subdir");

        assert_eq!(subdir_node.borrow().name, "subdir");
        assert_eq!(subdir_node.borrow().path, subdir_path);
        assert_eq!(subdir_node.borrow().node_type, FsNodeType::Directory);

        // Try to find a non-existent file
        let non_existent_path = temp_dir.path().join("non_existent.txt");
        let non_existent_node = root_mut.find_node(non_existent_path, None);
        assert!(non_existent_node.is_none());
    }

    #[test]
    fn test_add_child() {
        let temp_dir = create_test_directory();

        // Create a node tree from the root directory
        let root_node = FsNode::create_node_from_path(temp_dir.path().to_path_buf(), None)
            .expect("Failed to create root node");

        // Create a new file in the temp directory
        let new_file_path = temp_dir.path().join("new_file.txt");
        let mut new_file = fs::File::create(&new_file_path).expect("Failed to create new_file.txt");
        new_file.write_all(b"New file content").expect("Failed to write to new_file.txt");

        // Create a new node for the file
        let new_file_node = FsNode::new(
            "new_file.txt".to_string(),
            new_file_path.clone(),
            FsNodeType::File,
            None,
            vec![]
        );

        // Add the new file node as a child of the root node
        FsNode::add_child(&root_node, new_file_node);

        // Check that the new file node is in the children of the root node
        let root = root_node.borrow();
        let new_file_found = root.children.iter().any(|child| {
            let child = child.borrow();
            child.name == "new_file.txt" && child.path == new_file_path
        });

        assert!(new_file_found, "New file node not found in children");

        // Check that the parent of the new file node is the root node
        let new_file_node = root.children.iter()
            .find(|child| child.borrow().name == "new_file.txt")
            .expect("New file node not found");

        let new_file_parent = new_file_node.borrow().parent.as_ref()
            .and_then(|weak| weak.upgrade())
            .expect("Failed to get parent of new file");

        assert_eq!(new_file_parent.borrow().path, root.path);
    }

    #[test]
    fn test_remove_node() {
        let temp_dir = create_test_directory();

        // Create a node tree from the root directory
        let root_node = FsNode::create_node_from_path(temp_dir.path().to_path_buf(), None)
            .expect("Failed to create root node");

        let file1_path = temp_dir.path().join("file1.txt");

        // Count initial children
        let initial_children_count = root_node.borrow().children.len();

        // Remove file1.txt
        let removed_node = root_node.borrow_mut()
            .remove_node(file1_path.clone(), Some(FsNodeType::File));

        assert!(removed_node.is_some(), "Node should have been removed");
        let removed = removed_node.unwrap();
        assert_eq!(removed.borrow().name, "file1.txt");
        assert_eq!(removed.borrow().path, file1_path);

        // Check that the number of children decreased
        assert_eq!(root_node.borrow().children.len(), initial_children_count - 1);

        // Try to find the removed node (should fail)
        let mut root_mut = root_node.borrow_mut();
        let find_result = root_mut.find_node(file1_path, Some(FsNodeType::File));
        assert!(find_result.is_none(), "Removed node should not be found");
    }

    #[test]
    fn test_deep_directory_structure() {
        let temp_dir = create_test_directory();

        // Access the nested directory path
        let nested_path = temp_dir.path().join("subdir").join("nested");

        // Create a node directly from the nested directory
        let nested_node = FsNode::create_node_from_path(nested_path.clone(), None)
            .expect("Failed to create nested node");

        // Check nested node properties
        let nested = nested_node.borrow();
        assert_eq!(nested.name, "nested");
        assert_eq!(nested.path, nested_path);
        assert_eq!(nested.node_type, FsNodeType::Directory);

        // Nested should have 1 child (file4.txt)
        assert_eq!(nested.children.len(), 1);

        // Check file4.txt
        let file4_node = &nested.children[0];
        let file4 = file4_node.borrow();
        assert_eq!(file4.name, "file4.txt");
        assert_eq!(file4.path, nested_path.join("file4.txt"));
        assert_eq!(file4.node_type, FsNodeType::File);

        // Check that file4's parent is the nested directory
        let file4_parent = file4.parent.as_ref()
            .and_then(|weak| weak.upgrade())
            .expect("Failed to get parent of file4");

        assert_eq!(file4_parent.borrow().path, nested.path);
    }
}