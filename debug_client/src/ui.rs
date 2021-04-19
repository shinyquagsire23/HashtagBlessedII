use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{
        Block, Borders,
        Paragraph, Sparkline, Wrap,
    },
    Frame,
};
use crate::{get_log_buf, get_sparkline_max, get_sparkline};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Min(0)].as_ref())
        .split(f.size());

    draw_first_tab(f, app, chunks[0]);
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(7),
                Constraint::Min(8),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);
    draw_gauges(f, app, chunks[0]);
    draw_charts(f, app, chunks[1]);
    draw_text(f, app, chunks[2]);
}

fn draw_gauges<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(3),
                //Constraint::Length(1),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(area);
    let block = Block::default().borders(Borders::ALL).title(format!("Sparkline (cur {}ns max {}ns):", get_sparkline(), get_sparkline_max()));
    f.render_widget(block, area);

    let sparkline = Sparkline::default()
        .block(Block::default())
        .style(Style::default().fg(Color::Green))
        .data(&app.sparkline.points)
        .max(get_sparkline_max())
        .bar_set(if app.enhanced_graphics {
            symbols::bar::NINE_LEVELS
        } else {
            symbols::bar::THREE_LEVELS
        });
    f.render_widget(sparkline, chunks[0]);

    /*let line_gauge = LineGauge::default()
        .block(Block::default().title("LineGauge:"))
        .gauge_style(Style::default().fg(Color::Magenta))
        .line_set(if app.enhanced_graphics {
            symbols::line::THICK
        } else {
            symbols::line::NORMAL
        })
        .ratio(app.progress);
    f.render_widget(line_gauge, chunks[1]);*/
}

fn draw_charts<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let constraints = vec![Constraint::Percentage(100)];
    let chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Horizontal)
        .split(area);
    {

        let log_buf_split = get_log_buf().as_str().lines();
        let text: Vec<Spans> = log_buf_split.map(|a| Spans::from(format!("{}", a))).collect();
        
        let block_h = (area.height as i32) - 2;
        let scroll_max = (text.len() as i32) - block_h;
        let mut scroll_val = scroll_max - app.scroll_up;
        if scroll_val < 0 {
            scroll_val = 0;
        }
        if app.scroll_up > scroll_max && scroll_max > 0 {
            app.scroll_up = scroll_max;
        }
        
        let the_block = Block::default().borders(Borders::ALL).title("Log Output");
        
        let logs = Paragraph::new(text).block(the_block).wrap(Wrap { trim: false }).scroll((scroll_val as u16, 0));
        f.render_widget(logs, chunks[0]);
    }
}

fn draw_text<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let spin = ["|", "/", "-", "\\"];
    let spin_idx = ((app.ticks / 6) & 3) as usize;

    let is_blink = ((app.ticks / 20) & 1) == 0;
    let mut cmd_split0 = app.cmdbuf.clone();
    let mut cmd_split1 = cmd_split0.split_off(app.cursor_idx);
    let mut cmd_split_c = if is_blink { '⠀' } else { ' ' };
    if !cmd_split1.is_empty() {
        cmd_split_c = cmd_split1.remove(0);
    }
    if cmd_split1.is_empty() {
        cmd_split1 = format!(" ");
    }

    let text = vec![
        Spans::from(vec![
            Span::from(format!("{} > {}", spin[spin_idx], cmd_split0)),
            if is_blink { Span::styled(format!("{}", cmd_split_c), Style::default().bg(Color::LightBlue).fg(Color::Black)) } else { Span::from(format!("{}", cmd_split_c)) },
            Span::from(format!("{}", cmd_split1)),
        ])
    ];
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
