pub fn apply_move(fen: &str, mv: &str) -> String {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    if parts.len() < 4 { return fen.to_string(); }

    let mut board = parse_board(parts[0]);
    let side_white = parts[1] == "w";
    let mut castling = parts[2].to_string();
    let mut en_passant = "-".to_string();
    let halfmove: u32 = parts[4].parse().unwrap_or(0);
    let fullmove: u32 = parts[5].parse().unwrap_or(1);

    let from = sq_from_str(&mv[0..2]);
    let to   = sq_from_str(&mv[2..4]);
    let promo = mv.as_bytes().get(4).copied();

    let piece = board[from];
    let captured = board[to];

    let is_pawn = piece.map_or(false, |(_, p)| p == 0);
    let is_king = piece.map_or(false, |(_, p)| p == 5);
    let is_rook = piece.map_or(false, |(_, p)| p == 3);

    let ep_sq: Option<usize> = if parts[3] != "-" { Some(sq_from_str(parts[3])) } else { None };

    let is_ep = is_pawn && Some(to) == ep_sq;
    if is_ep {
        let cap_sq = if side_white { to - 8 } else { to + 8 };
        board[cap_sq] = None;
    }

    if is_king {
        let (from_file, to_file) = (from % 8, to % 8);
        if from_file == 4 && to_file == 6 {
            let rook_from = if side_white { 7 } else { 56 + 7 };
            let rook_to   = if side_white { 5 } else { 56 + 5 };
            board[rook_to] = board[rook_from];
            board[rook_from] = None;
        } else if from_file == 4 && to_file == 2 {
            let rook_from = if side_white { 0 } else { 56 };
            let rook_to   = if side_white { 3 } else { 56 + 3 };
            board[rook_to] = board[rook_from];
            board[rook_from] = None;
        }
        if side_white {
            castling = castling.replace('K', "").replace('Q', "");
        } else {
            castling = castling.replace('k', "").replace('q', "");
        }
        if castling.is_empty() { castling = "-".to_string(); }
    }

    if is_rook {
        match from {
            0  => castling = castling.replace('Q', ""),
            7  => castling = castling.replace('K', ""),
            56 => castling = castling.replace('q', ""),
            63 => castling = castling.replace('k', ""),
            _  => {}
        }
        if castling.is_empty() { castling = "-".to_string(); }
    }

    board[to] = if let Some(p) = promo {
        let piece_idx = match p {
            b'q' => 4, b'r' => 3, b'b' => 2, b'n' => 1, _ => 4,
        };
        Some((side_white, piece_idx))
    } else {
        piece
    };
    board[from] = None;

    if is_pawn && (to as i32 - from as i32).abs() == 16 {
        let ep = if side_white { from + 8 } else { from - 8 };
        en_passant = sq_to_str(ep);
    }

    let new_halfmove = if is_pawn || captured.is_some() { 0 } else { halfmove + 1 };
    let new_fullmove = if side_white { fullmove } else { fullmove + 1 };
    let new_side = if side_white { "b" } else { "w" };

    format!(
        "{} {} {} {} {} {}",
        board_to_fen(&board),
        new_side,
        castling,
        en_passant,
        new_halfmove,
        new_fullmove,
    )
}

fn parse_board(s: &str) -> [Option<(bool, u8)>; 64] {
    let mut board = [None; 64];
    let mut sq: usize = 56;
    for ch in s.chars() {
        match ch {
            '/' => { sq = sq.saturating_sub(16); }
            '1'..='8' => { sq += (ch as u8 - b'0') as usize; }
            _ => {
                let white = ch.is_uppercase();
                let p = match ch.to_ascii_lowercase() {
                    'p' => 0, 'n' => 1, 'b' => 2, 'r' => 3, 'q' => 4, 'k' => 5, _ => { sq += 1; continue; }
                };
                if sq < 64 { board[sq] = Some((white, p)); }
                sq += 1;
            }
        }
    }
    board
}

fn board_to_fen(board: &[Option<(bool, u8)>; 64]) -> String {
    let mut s = String::new();
    for rank in (0..8).rev() {
        let mut empty = 0u8;
        for file in 0..8 {
            let sq = rank * 8 + file;
            match board[sq] {
                None => { empty += 1; }
                Some((white, piece)) => {
                    if empty > 0 { s.push((b'0' + empty) as char); empty = 0; }
                    let ch = match piece {
                        0 => 'p', 1 => 'n', 2 => 'b', 3 => 'r', 4 => 'q', 5 => 'k', _ => '?',
                    };
                    s.push(if white { ch.to_ascii_uppercase() } else { ch });
                }
            }
        }
        if empty > 0 { s.push((b'0' + empty) as char); }
        if rank > 0 { s.push('/'); }
    }
    s
}

pub fn sq_from_str(s: &str) -> usize {
    let b = s.as_bytes();
    let file = (b[0] - b'a') as usize;
    let rank = (b[1] - b'1') as usize;
    rank * 8 + file
}

fn sq_to_str(sq: usize) -> String {
    format!("{}{}", (b'a' + (sq % 8) as u8) as char, (b'1' + (sq / 8) as u8) as char)
}