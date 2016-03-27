//! `Dyn` impls for various types of widgets.  These combine widget-specific state (from
//! `ui::state`) with external data (such as `client.inventories`).

use physics::v3::{V2, scalar, Region};

use inventory::{Inventory, Inventories, Item};
use ui::{dialog, hotbar, inventory, item, root};
use ui::widget::*;

