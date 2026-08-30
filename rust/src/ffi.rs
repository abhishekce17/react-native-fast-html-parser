use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use crate::models::{ContentBlock, DefinitionItem, InlineNode, ListItem, TableCell, TableRow};
use crate::parse_html;

// Opaque container to pass across the FFI boundary
pub struct ParsedArticle {
    pub blocks: Vec<ContentBlock>,
}

#[unsafe(no_mangle)]
pub extern "C" fn parse_html_ffi(html_ptr: *const c_char) -> *mut ParsedArticle {
    if html_ptr.is_null() { return std::ptr::null_mut(); }
    
    let c_str = unsafe { CStr::from_ptr(html_ptr) };
    let html = c_str.to_string_lossy();
    
    let blocks = parse_html(&html);
    Box::into_raw(Box::new(ParsedArticle { blocks }))
}

#[unsafe(no_mangle)]
pub extern "C" fn free_article_ffi(article: *mut ParsedArticle) {
    if !article.is_null() {
        unsafe { let _ = Box::from_raw(article); }
    }
}

// ── Block Getters ──────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn get_block_count(article: *const ParsedArticle) -> usize {
    if article.is_null() { return 0; }
    unsafe { (*article).blocks.len() }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_block_type(article: *const ParsedArticle, index: usize) -> *mut c_char {
    if article.is_null() { return std::ptr::null_mut(); }
    let blocks = unsafe { &(*article).blocks };
    if index >= blocks.len() { return std::ptr::null_mut(); }

    let type_str = match &blocks[index] {
        ContentBlock::Paragraph { .. } => "Paragraph",
        ContentBlock::Heading { .. } => "Heading",
        ContentBlock::Image { .. } => "Image",
        ContentBlock::Figure { .. } => "Figure",
        ContentBlock::CodeBlock { .. } => "CodeBlock",
        ContentBlock::List { .. } => "List",
        ContentBlock::Table { .. } => "Table",
        ContentBlock::Quote { .. } => "Quote",
        ContentBlock::DefinitionList { .. } => "DefinitionList",
        ContentBlock::Video { .. } => "Video",
        ContentBlock::Audio { .. } => "Audio",
        ContentBlock::Embed { .. } => "Embed",
        ContentBlock::Separator { .. } => "Separator",
    };

    CString::new(type_str).unwrap().into_raw()
}

// ── Field Getters ───────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn get_heading_level(article: *const ParsedArticle, index: usize) -> u8 {
    if article.is_null() { return 0; }
    let blocks = unsafe { &(*article).blocks };
    if index >= blocks.len() { return 0; }

    match &blocks[index] {
        ContentBlock::Heading { level, .. } => *level,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_image_url(article: *const ParsedArticle, index: usize) -> *mut c_char {
    if article.is_null() { return std::ptr::null_mut(); }
    let blocks = unsafe { &(*article).blocks };
    if index >= blocks.len() { return std::ptr::null_mut(); }

    let url = match &blocks[index] {
        ContentBlock::Image { url, .. } => url.clone(),
        ContentBlock::Figure { url, .. } => url.clone(),
        _ => "".to_string(),
    };

    CString::new(url).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_image_alt(article: *const ParsedArticle, index: usize) -> *mut c_char {
    if article.is_null() { return std::ptr::null_mut(); }
    let blocks = unsafe { &(*article).blocks };
    if index >= blocks.len() { return std::ptr::null_mut(); }

    let alt = match &blocks[index] {
        ContentBlock::Image { alt, .. } => alt.clone().unwrap_or_default(),
        ContentBlock::Figure { alt, .. } => alt.clone().unwrap_or_default(),
        _ => "".to_string(),
    };

    CString::new(alt).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_codeblock_code(article: *const ParsedArticle, index: usize) -> *mut c_char {
    if article.is_null() { return std::ptr::null_mut(); }
    let blocks = unsafe { &(*article).blocks };
    if index >= blocks.len() { return std::ptr::null_mut(); }

    let code = match &blocks[index] {
        ContentBlock::CodeBlock { code, .. } => code.clone(),
        _ => "".to_string(),
    };

    CString::new(code).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_codeblock_lang(article: *const ParsedArticle, index: usize) -> *mut c_char {
    if article.is_null() { return std::ptr::null_mut(); }
    let blocks = unsafe { &(*article).blocks };
    if index >= blocks.len() { return std::ptr::null_mut(); }

    let lang = match &blocks[index] {
        ContentBlock::CodeBlock { language, .. } => language.clone().unwrap_or_default(),
        _ => "".to_string(),
    };

    CString::new(lang).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_block_child_count(article: *const ParsedArticle, index: usize) -> usize {
    if article.is_null() { return 0; }
    let blocks = unsafe { &(*article).blocks };
    if index >= blocks.len() { return 0; }

    match &blocks[index] {
        ContentBlock::Paragraph { children } => children.len(),
        ContentBlock::Heading { children, .. } => children.len(),
        _ => 0,
    }
}

// ── ContentBlock Pointer Getters ─────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn get_block_by_index(article: *const ParsedArticle, index: usize) -> *const ContentBlock {
    if article.is_null() { return std::ptr::null(); }
    let blocks = unsafe { &(*article).blocks };
    if index >= blocks.len() { return std::ptr::null(); }
    &blocks[index] as *const ContentBlock
}

#[unsafe(no_mangle)]
pub extern "C" fn get_block_type_ptr(block: *const ContentBlock) -> *mut c_char {
    if block.is_null() { return std::ptr::null_mut(); }
    let block_ref = unsafe { &*block };
    let type_str = match block_ref {
        ContentBlock::Paragraph { .. } => "Paragraph",
        ContentBlock::Heading { .. } => "Heading",
        ContentBlock::Image { .. } => "Image",
        ContentBlock::Figure { .. } => "Figure",
        ContentBlock::CodeBlock { .. } => "CodeBlock",
        ContentBlock::List { .. } => "List",
        ContentBlock::Table { .. } => "Table",
        ContentBlock::Quote { .. } => "Quote",
        ContentBlock::DefinitionList { .. } => "DefinitionList",
        ContentBlock::Video { .. } => "Video",
        ContentBlock::Audio { .. } => "Audio",
        ContentBlock::Embed { .. } => "Embed",
        ContentBlock::Separator { .. } => "Separator",
    };
    CString::new(type_str).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_heading_level_ptr(block: *const ContentBlock) -> u8 {
    if block.is_null() { return 0; }
    let block_ref = unsafe { &*block };
    match block_ref {
        ContentBlock::Heading { level, .. } => *level,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_image_url_ptr(block: *const ContentBlock) -> *mut c_char {
    if block.is_null() { return std::ptr::null_mut(); }
    let block_ref = unsafe { &*block };
    let url = match block_ref {
        ContentBlock::Image { url, .. } => url.clone(),
        ContentBlock::Figure { url, .. } => url.clone(),
        _ => "".to_string(),
    };
    CString::new(url).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_image_alt_ptr(block: *const ContentBlock) -> *mut c_char {
    if block.is_null() { return std::ptr::null_mut(); }
    let block_ref = unsafe { &*block };
    let alt = match block_ref {
        ContentBlock::Image { alt, .. } => alt.clone().unwrap_or_default(),
        ContentBlock::Figure { alt, .. } => alt.clone().unwrap_or_default(),
        _ => "".to_string(),
    };
    CString::new(alt).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_codeblock_code_ptr(block: *const ContentBlock) -> *mut c_char {
    if block.is_null() { return std::ptr::null_mut(); }
    let block_ref = unsafe { &*block };
    let code = match block_ref {
        ContentBlock::CodeBlock { code, .. } => code.clone(),
        _ => "".to_string(),
    };
    CString::new(code).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_codeblock_lang_ptr(block: *const ContentBlock) -> *mut c_char {
    if block.is_null() { return std::ptr::null_mut(); }
    let block_ref = unsafe { &*block };
    let lang = match block_ref {
        ContentBlock::CodeBlock { language, .. } => language.clone().unwrap_or_default(),
        _ => "".to_string(),
    };
    CString::new(lang).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_video_src_ptr(block: *const ContentBlock) -> *mut c_char {
    if block.is_null() { return std::ptr::null_mut(); }
    let block_ref = unsafe { &*block };
    let src = match block_ref {
        ContentBlock::Video { src, .. } => src.clone(),
        _ => "".to_string(),
    };
    CString::new(src).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_video_poster_ptr(block: *const ContentBlock) -> *mut c_char {
    if block.is_null() { return std::ptr::null_mut(); }
    let block_ref = unsafe { &*block };
    let poster = match block_ref {
        ContentBlock::Video { poster, .. } => poster.clone().unwrap_or_default(),
        _ => "".to_string(),
    };
    CString::new(poster).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_audio_src_ptr(block: *const ContentBlock) -> *mut c_char {
    if block.is_null() { return std::ptr::null_mut(); }
    let block_ref = unsafe { &*block };
    let src = match block_ref {
        ContentBlock::Audio { src } => src.clone(),
        _ => "".to_string(),
    };
    CString::new(src).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_embed_src_ptr(block: *const ContentBlock) -> *mut c_char {
    if block.is_null() { return std::ptr::null_mut(); }
    let block_ref = unsafe { &*block };
    let src = match block_ref {
        ContentBlock::Embed { src, .. } => src.clone(),
        _ => "".to_string(),
    };
    CString::new(src).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_embed_title_ptr(block: *const ContentBlock) -> *mut c_char {
    if block.is_null() { return std::ptr::null_mut(); }
    let block_ref = unsafe { &*block };
    let title = match block_ref {
        ContentBlock::Embed { title, .. } => title.clone().unwrap_or_default(),
        _ => "".to_string(),
    };
    CString::new(title).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_figure_caption_ptr(block: *const ContentBlock) -> *mut c_char {
    if block.is_null() { return std::ptr::null_mut(); }
    let block_ref = unsafe { &*block };
    let caption = match block_ref {
        ContentBlock::Figure { caption, .. } => caption.clone().unwrap_or_default(),
        _ => "".to_string(),
    };
    CString::new(caption).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_block_child_count_ptr(block: *const ContentBlock) -> usize {
    if block.is_null() { return 0; }
    let block_ref = unsafe { &*block };
    match block_ref {
        ContentBlock::Paragraph { children } => children.len(),
        ContentBlock::Heading { children, .. } => children.len(),
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_block_child_by_index(block: *const ContentBlock, index: usize) -> *const InlineNode {
    if block.is_null() { return std::ptr::null(); }
    let block_ref = unsafe { &*block };
    let children = match block_ref {
        ContentBlock::Paragraph { children } => children,
        ContentBlock::Heading { children, .. } => children,
        _ => return std::ptr::null(),
    };
    if index >= children.len() { return std::ptr::null(); }
    &children[index] as *const InlineNode
}

#[unsafe(no_mangle)]
pub extern "C" fn get_quote_child_count(block: *const ContentBlock) -> usize {
    if block.is_null() { return 0; }
    let block_ref = unsafe { &*block };
    match block_ref {
        ContentBlock::Quote { children } => children.len(),
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_quote_child_by_index(block: *const ContentBlock, index: usize) -> *const ContentBlock {
    if block.is_null() { return std::ptr::null(); }
    let block_ref = unsafe { &*block };
    match block_ref {
        ContentBlock::Quote { children } => {
            if index >= children.len() { return std::ptr::null(); }
            &children[index] as *const ContentBlock
        }
        _ => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_list_ordered(block: *const ContentBlock) -> bool {
    if block.is_null() { return false; }
    let block_ref = unsafe { &*block };
    match block_ref {
        ContentBlock::List { ordered, .. } => *ordered,
        _ => false,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_list_item_count(block: *const ContentBlock) -> usize {
    if block.is_null() { return 0; }
    let block_ref = unsafe { &*block };
    match block_ref {
        ContentBlock::List { items, .. } => items.len(),
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_list_item_by_index(block: *const ContentBlock, index: usize) -> *const ListItem {
    if block.is_null() { return std::ptr::null(); }
    let block_ref = unsafe { &*block };
    match block_ref {
        ContentBlock::List { items, .. } => {
            if index >= items.len() { return std::ptr::null(); }
            &items[index] as *const ListItem
        }
        _ => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_table_row_count(block: *const ContentBlock) -> usize {
    if block.is_null() { return 0; }
    let block_ref = unsafe { &*block };
    match block_ref {
        ContentBlock::Table { rows } => rows.len(),
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_table_row_by_index(block: *const ContentBlock, index: usize) -> *const TableRow {
    if block.is_null() { return std::ptr::null(); }
    let block_ref = unsafe { &*block };
    match block_ref {
        ContentBlock::Table { rows } => {
            if index >= rows.len() { return std::ptr::null(); }
            &rows[index] as *const TableRow
        }
        _ => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_def_list_item_count(block: *const ContentBlock) -> usize {
    if block.is_null() { return 0; }
    let block_ref = unsafe { &*block };
    match block_ref {
        ContentBlock::DefinitionList { items } => items.len(),
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_def_list_item_by_index(block: *const ContentBlock, index: usize) -> *const DefinitionItem {
    if block.is_null() { return std::ptr::null(); }
    let block_ref = unsafe { &*block };
    match block_ref {
        ContentBlock::DefinitionList { items } => {
            if index >= items.len() { return std::ptr::null(); }
            &items[index] as *const DefinitionItem
        }
        _ => std::ptr::null(),
    }
}

// ── ListItem Pointer Getters ─────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn get_list_item_child_count(item: *const ListItem) -> usize {
    if item.is_null() { return 0; }
    let item_ref = unsafe { &*item };
    match item_ref {
        ListItem::Item { children, .. } => children.len(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_list_item_child_by_index(item: *const ListItem, index: usize) -> *const InlineNode {
    if item.is_null() { return std::ptr::null(); }
    let item_ref = unsafe { &*item };
    match item_ref {
        ListItem::Item { children, .. } => {
            if index >= children.len() { return std::ptr::null(); }
            &children[index] as *const InlineNode
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_list_item_nested_count(item: *const ListItem) -> usize {
    if item.is_null() { return 0; }
    let item_ref = unsafe { &*item };
    match item_ref {
        ListItem::Item { nested, .. } => nested.len(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_list_item_nested_by_index(item: *const ListItem, index: usize) -> *const ContentBlock {
    if item.is_null() { return std::ptr::null(); }
    let item_ref = unsafe { &*item };
    match item_ref {
        ListItem::Item { nested, .. } => {
            if index >= nested.len() { return std::ptr::null(); }
            &nested[index] as *const ContentBlock
        }
    }
}

// ── TableRow Pointer Getters ─────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn get_table_row_cell_count(row: *const TableRow) -> usize {
    if row.is_null() { return 0; }
    let row_ref = unsafe { &*row };
    match row_ref {
        TableRow::Row { cells } => cells.len(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_table_row_cell_by_index(row: *const TableRow, index: usize) -> *const TableCell {
    if row.is_null() { return std::ptr::null(); }
    let row_ref = unsafe { &*row };
    match row_ref {
        TableRow::Row { cells } => {
            if index >= cells.len() { return std::ptr::null(); }
            &cells[index] as *const TableCell
        }
    }
}

// ── TableCell Pointer Getters ────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn get_table_cell_child_count(cell: *const TableCell) -> usize {
    if cell.is_null() { return 0; }
    let cell_ref = unsafe { &*cell };
    match cell_ref {
        TableCell::Cell { children } => children.len(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_table_cell_child_by_index(cell: *const TableCell, index: usize) -> *const InlineNode {
    if cell.is_null() { return std::ptr::null(); }
    let cell_ref = unsafe { &*cell };
    match cell_ref {
        TableCell::Cell { children } => {
            if index >= children.len() { return std::ptr::null(); }
            &children[index] as *const InlineNode
        }
    }
}

// ── DefinitionItem Pointer Getters ───────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn get_def_item_term_count(item: *const DefinitionItem) -> usize {
    if item.is_null() { return 0; }
    let item_ref = unsafe { &*item };
    item_ref.term.len()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_def_item_term_by_index(item: *const DefinitionItem, index: usize) -> *const InlineNode {
    if item.is_null() { return std::ptr::null(); }
    let item_ref = unsafe { &*item };
    if index >= item_ref.term.len() { return std::ptr::null(); }
    &item_ref.term[index] as *const InlineNode
}

#[unsafe(no_mangle)]
pub extern "C" fn get_def_item_def_count(item: *const DefinitionItem) -> usize {
    if item.is_null() { return 0; }
    let item_ref = unsafe { &*item };
    item_ref.definition.len()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_def_item_def_by_index(item: *const DefinitionItem, index: usize) -> *const InlineNode {
    if item.is_null() { return std::ptr::null(); }
    let item_ref = unsafe { &*item };
    if index >= item_ref.definition.len() { return std::ptr::null(); }
    &item_ref.definition[index] as *const InlineNode
}

// ── InlineNode Pointer Getters ───────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn get_inline_node_type(node: *const InlineNode) -> *mut c_char {
    if node.is_null() { return std::ptr::null_mut(); }
    let node_ref = unsafe { &*node };
    let type_str = match node_ref {
        InlineNode::Text { .. } => "Text",
        InlineNode::Bold { .. } => "Bold",
        InlineNode::Italic { .. } => "Italic",
        InlineNode::Link { .. } => "Link",
        InlineNode::InlineCode { .. } => "InlineCode",
        InlineNode::Break { .. } => "Break",
    };
    CString::new(type_str).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_inline_node_text(node: *const InlineNode) -> *mut c_char {
    if node.is_null() { return std::ptr::null_mut(); }
    let node_ref = unsafe { &*node };
    let text = match node_ref {
        InlineNode::Text { text } => text.clone(),
        InlineNode::InlineCode { text } => text.clone(),
        _ => "".to_string(),
    };
    CString::new(text).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_inline_node_url(node: *const InlineNode) -> *mut c_char {
    if node.is_null() { return std::ptr::null_mut(); }
    let node_ref = unsafe { &*node };
    let url = match node_ref {
        InlineNode::Link { url, .. } => url.clone(),
        _ => "".to_string(),
    };
    CString::new(url).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_inline_node_child_count(node: *const InlineNode) -> usize {
    if node.is_null() { return 0; }
    let node_ref = unsafe { &*node };
    match node_ref {
        InlineNode::Bold { children } => children.len(),
        InlineNode::Italic { children } => children.len(),
        InlineNode::Link { children, .. } => children.len(),
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_inline_node_child_by_index(node: *const InlineNode, index: usize) -> *const InlineNode {
    if node.is_null() { return std::ptr::null(); }
    let node_ref = unsafe { &*node };
    let children = match node_ref {
        InlineNode::Bold { children } => children,
        InlineNode::Italic { children } => children,
        InlineNode::Link { children, .. } => children,
        _ => return std::ptr::null(),
    };
    if index >= children.len() { return std::ptr::null(); }
    &children[index] as *const InlineNode
}

// ── Clean Up ─────────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn free_string_ffi(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}
