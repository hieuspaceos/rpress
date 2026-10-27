<?php
	/*	
	*	Goodlayers Menu Management File
	*	---------------------------------------------------------------------
	*	This file modify the menu area for mega menu implementation
	*	---------------------------------------------------------------------
	*/
	
	if( !class_exists('gdlr_core_edit_nav_menu') ){
		class gdlr_core_edit_nav_menu{
			
			private $options = array();
			private $option_slug = 'gdlr_core_nav_menu_custom';
			
			function __construct( $options ){
				
				$this->options = $options;

				// check for wp5.4
				if( function_exists('apply_shortcodes') ){
					
					add_action('wp_nav_menu_item_custom_fields', array(&$this, 'get_custom_nav_menu_fields'), 10, 4);
				
				}else{

					// rewrite the edit nav walker
					add_filter('wp_edit_nav_menu_walker', array(&$this, 'get_edit_nav_walker'), 10, 2);

					// add action to display fields
					add_action('gdlr_core_walker_nav_menu_edit_fields', array(&$this, 'get_nav_menu_fields'));
					
					// register the option slug to
					add_filter('gdlr_core_nav_menu_import_slug', array(&$this, 'nav_menu_import_slug'));
				}

				// add the script when opening the menu page
				add_action('admin_enqueue_scripts', array(&$this, 'load_menu_script') );

				// setup menu data
				add_filter('wp_setup_nav_menu_item', array(&$this, 'setup_nav_menu'));
				
				// add action to save the data
				add_action('wp_update_nav_menu_item',  array(&$this, 'save_nav_menu_item'), 10, 3 );
					
			}

			// custom nav menu fields
			function get_custom_nav_menu_fields( $id, $item, $depth, $args ) {
			   	
			   	$this->get_nav_menu_fields($item);

			}
			
			// load the necessary script for the menu page
			function load_menu_script( $hook ){
				
				if( $hook == 'nav-menus.php' ){
					wp_enqueue_media();

					wp_enqueue_style('wp-color-picker');
					wp_enqueue_style('gdlr-core-navigation-menu', GDLR_CORE_URL . '/framework/css/navigation-menu.css');
					
					wp_enqueue_script('gdlr-core-navigation-menu', GDLR_CORE_URL . '/framework/js/navigation-menu.js', array('jquery', 'wp-color-picker'), false, true);
				}
				
			}
			
			// define the backend walker menu
			function get_edit_nav_walker($walker, $menu_id){
				return 'gdlr_core_walker_nav_menu_edit';
			}
			
			// initialize the data of the nav menu
			function setup_nav_menu($menu_item){
				$menu_item->gdlr_core_nav_menu_custom = get_post_meta($menu_item->ID, $this->option_slug, true);
				return $menu_item;
			}
			
			// function to save nav menu item
			function save_nav_menu_item($menu_id, $menu_item_db_id, $args){
				if( isset($_REQUEST[$this->option_slug]) && is_array($_REQUEST[$this->option_slug]) ){
					$nav_menu_custom = json_decode(gdlr_core_process_post_data($_REQUEST[$this->option_slug][$menu_item_db_id]), true);
					update_post_meta($menu_item_db_id, $this->option_slug, $nav_menu_custom);				
				}		
			}
			
			// get field for displaying in menu page
			function get_nav_menu_fields( $item ){
				
				// init the value
				$value = $item->gdlr_core_nav_menu_custom;
				if( empty($value) ){
					$value = array();
					foreach($this->options as $option_slug => $option){
						if( !empty($option['default']) ){
							$value[$option_slug] = $option['default'];
						}
					}
				}
				
				// print field from option
				echo '<div class="gdlr-core-custom-nav-menu-fields" >';
				foreach($this->options as $option_slug => $option){
					$option_id = 'edit-menu-item-' . $option_slug . '-' . $item->ID;
					$option_val = empty($value[$option_slug])? '': $value[$option_slug];
?>
<div class="clear"></div>
<div class="gdlr-core-custom-nav-menu-field gdlr-core-custom-nav-menu-field-<?php echo esc_attr($option['type']); ?>" <?php echo isset($option['depth'])? 'data-depth="' . esc_attr($option['depth']) . '"': ''; ?> > 
	<label for="<?php echo esc_attr($option_id); ?>">
		<?php 
			switch( $option['type'] ){
				case 'text' : 
					echo gdlr_core_escape_content($option['title']) . '<br />';
					echo '<input type="text" id="' . esc_attr($option_id) . '" data-slug="' . esc_attr($option_slug) . '" value="' . esc_attr($option_val) . '" />';
					break;

				case 'colorpicker' : 
					echo gdlr_core_escape_content($option['title']) . '<br />';
					echo '<div><input class="gdlr-core-colorpicker" type="text" id="' . esc_attr($option_id) . '" data-slug="' . esc_attr($option_slug) . '" value="' . esc_attr($option_val) . '" /></div>';
					break;
					
				case 'textarea' : 
					echo gdlr_core_escape_content($option['title']) . '<br />';
					echo '<textarea id="' . esc_attr($option_id) . '" data-slug="' . esc_attr($option_slug) . '" >' . esc_textarea($option_val) . '</textarea>';
					break;
					
				case 'checkbox' : 
					echo '<input type="checkbox" id="' . esc_attr($option_id) . '" data-slug="' . esc_attr($option_slug) . '" ' . checked($option_val, 'enable', false) . ' /> ';
					echo gdlr_core_escape_content($option['title']);
					break;

				case 'upload' : 
					echo gdlr_core_escape_content($option['title']) . '<br />';

					$thumbnail = empty($option_val)? '': gdlr_core_get_image_url($option_val, 'thumbnail', false);
					echo '<img class="gdlr-core-upload-input-img" src="' . esc_attr($thumbnail) . '" />';
					echo '<input class="gdlr-core-upload-input" type="hidden" id="' . esc_attr($option_id) . '" data-slug="' . esc_attr($option_slug) . '" value="' . esc_attr($option_val) . '" /> ';
					echo '<input class="gdlr-core-upload-button" type="button" value="' . esc_html__('Upload', 'goodlayers-core') . '" />';
					echo '<input class="gdlr-core-upload-button-remove ' . (empty($thumbnail)? '': 'gdlr-core-active') . '" type="button" value="' . esc_html__('Remove Image', 'goodlayers-core') . '" />';
					
					break;
					
				case 'combobox' : 
					echo gdlr_core_escape_content($option['title']) . '<br />';
					echo '<select id="' . esc_attr($option_id) . '" data-slug="' . esc_attr($option_slug) . '" >';
					foreach( $option['options'] as $slug => $val ){
						echo '<option value="' . esc_attr($slug) . '" ' . selected($option_val, $slug, true) . ' >' . gdlr_core_escape_content($val) . '</option>';
					}
					echo '</select>';
					break;
					
				default: break; 
			}
		?>
	</label>
</div>
<?php			
				}
				echo '<input class="gdlr-core-custom-nav-menu-val" type="hidden" name="' . esc_attr($this->option_slug) . '[' . esc_attr($item->ID) . ']" value="' . esc_attr(json_encode($value)) . '" />';
				echo '</div>'; // gdlr-core-custom-nav-menu-fields
			}

			// add the custom meta to array
			function nav_menu_import_slug($slugs){
				$slugs[] = $this->option_slug;
				return $slugs;
			}	

		} // gdlr_core_edit_nav_menu
	} // class_exists
	
	// create gdlr_core_walker_nav_menu_edit class to add the custom field
	// from wp-admin/includes/class-walker-nav-menu-edit.php file
	if( !class_exists('gdlr_core_walker_nav_menu_edit') ){
		class gdlr_core_walker_nav_menu_edit extends Walker_Nav_Menu{

			function start_lvl( &$output, $depth = 0, $args = array() ) {}

			function end_lvl( &$output, $depth = 0, $args = array() ) {}	
		
			function start_el( &$output, $item, $depth = 0, $args = array(), $id = 0 ) {
				global $_wp_nav_menu_max_depth;
				$_wp_nav_menu_max_depth = $depth > $_wp_nav_menu_max_depth ? $depth : $_wp_nav_menu_max_depth;

				ob_start();
				$item_id = esc_attr( $item->ID );
				$removed_args = array(
					'action',
					'customlink-tab',
					'edit-menu-item',
					'menu-item',
					'page-tab',
					'_wpnonce',
				);

				$original_title = '';
				if ( 'taxonomy' == $item->type ) {
					$original_title = get_term_field( 'name', $item->object_id, $item->object, 'raw' );
					if ( is_wp_error( $original_title ) )
						$original_title = false;
				} elseif ( 'post_type' == $item->type ) {
					$original_object = get_post( $item->object_id );
					$original_title = get_the_title( $original_object->ID );
				} elseif ( 'post_type_archive' == $item->type ) {
					$original_object = get_post_type_object( $item->object );
					$original_title = $original_object->labels->archives;
				}

				$classes = array(
					'menu-item menu-item-depth-' . $depth,
					'menu-item-' . esc_attr( $item->object ),
					'menu-item-edit-' . ( ( isset( $_GET['edit-menu-item'] ) && $item_id == $_GET['edit-menu-item'] ) ? 'active' : 'inactive'),
				);

				$title = $item->title;

				if ( ! empty( $item->_invalid ) ) {
					$classes[] = 'menu-item-invalid';
					/* translators: %s: title of menu item which is invalid */
					$title = sprintf( esc_html__( '%s (Invalid)', 'goodlayers-core' ), $item->title );
				} elseif ( isset( $item->post_status ) && 'draft' == $item->post_status ) {
					$classes[] = 'pending';
					/* translators: %s: title of menu item in draft status */
					$title = sprintf( esc_html__('%s (Pending)', 'goodlayers-core'), $item->title );
				}

				$title = ( ! isset( $item->label ) || '' == $item->label ) ? $title : $item->label;

				$submenu_text = '';
				if ( 0 == $depth )
					$submenu_text = 'style="display: none;"';

				?>
				<li id="menu-item-<?php echo esc_attr($item_id); ?>" class="<?php echo implode(' ', $classes ); ?>">
					<div class="menu-item-bar">
						<div class="menu-item-handle">
							<span class="item-title"><span class="menu-item-title"><?php echo esc_html( $title ); ?></span> <span class="is-submenu" <?php echo esc_attr($submenu_text); ?>><?php esc_html_e( 'sub item', 'goodlayers-core' ); ?></span></span>
							<span class="item-controls">
								<span class="item-type"><?php echo esc_html( $item->type_label ); ?></span>
								<span class="item-order hide-if-js">
									<a href="<?php
										echo wp_nonce_url(
											add_query_arg(
												array(
													'action' => 'move-up-menu-item',
													'menu-item' => $item_id,
												),
												remove_query_arg($removed_args, admin_url( 'nav-menus.php' ) )
											),
											'move-menu_item'
										);
									?>" class="item-move-up"><abbr title="<?php esc_attr_e('Move up', 'goodlayers-core'); ?>">&#8593;</abbr></a>
									|
									<a href="<?php
										echo wp_nonce_url(
											add_query_arg(
												array(
													'action' => 'move-down-menu-item',
													'menu-item' => $item_id,
												),
												remove_query_arg($removed_args, admin_url( 'nav-menus.php' ) )
											),
											'move-menu_item'
										);
									?>" class="item-move-down"><abbr title="<?php esc_attr_e('Move down', 'goodlayers-core'); ?>">&#8595;</abbr></a>
								</span>
								<a class="item-edit" id="edit-<?php echo esc_attr($item_id); ?>" title="<?php esc_attr_e('Edit Menu Item', 'goodlayers-core'); ?>" href="<?php
									echo ( isset( $_GET['edit-menu-item'] ) && $item_id == $_GET['edit-menu-item'] ) ? admin_url( 'nav-menus.php' ) : add_query_arg( 'edit-menu-item', $item_id, remove_query_arg( $removed_args, admin_url( 'nav-menus.php#menu-item-settings-' . $item_id ) ) );
								?>"><?php esc_html_e( 'Edit Menu Item', 'goodlayers-core' ); ?></a>
							</span>
						</div>
					</div>

					<div class="menu-item-settings" id="menu-item-settings-<?php echo esc_attr($item_id); ?>">
						<?php if ( 'custom' == $item->type ) : ?>
							<p class="field-url description description-wide">
								<label for="edit-menu-item-url-<?php echo esc_attr($item_id); ?>">
									<?php esc_html_e( 'URL', 'goodlayers-core' ); ?><br />
									<input type="text" id="edit-menu-item-url-<?php echo esc_attr($item_id); ?>" class="widefat code edit-menu-item-url" name="menu-item-url[<?php echo esc_attr($item_id); ?>]" value="<?php echo esc_attr( $item->url ); ?>" />
								</label>
							</p>
						<?php endif; ?>
						<p class="description description-wide">
							<label for="edit-menu-item-title-<?php echo esc_attr($item_id); ?>">
								<?php esc_html_e( 'Navigation Label', 'goodlayers-core' ); ?><br />
								<input type="text" id="edit-menu-item-title-<?php echo esc_attr($item_id); ?>" class="widefat edit-menu-item-title" name="menu-item-title[<?php echo esc_attr($item_id); ?>]" value="<?php echo esc_attr( $item->title ); ?>" />
							</label>
						</p>
						<p class="field-title-attribute description description-wide">
							<label for="edit-menu-item-attr-title-<?php echo esc_attr($item_id); ?>">
								<?php esc_html_e( 'Title Attribute', 'goodlayers-core' ); ?><br />
								<input type="text" id="edit-menu-item-attr-title-<?php echo esc_attr($item_id); ?>" class="widefat edit-menu-item-attr-title" name="menu-item-attr-title[<?php echo esc_attr($item_id); ?>]" value="<?php echo esc_attr( $item->post_excerpt ); ?>" />
							</label>
						</p>
						<p class="field-link-target description">
							<label for="edit-menu-item-target-<?php echo esc_attr($item_id); ?>">
								<input type="checkbox" id="edit-menu-item-target-<?php echo esc_attr($item_id); ?>" value="_blank" name="menu-item-target[<?php echo esc_attr($item_id); ?>]"<?php checked( $item->target, '_blank' ); ?> />
								<?php esc_html_e( 'Open link in a new tab', 'goodlayers-core' ); ?>
							</label>
						</p>
						<p class="field-css-classes description description-thin">
							<label for="edit-menu-item-classes-<?php echo esc_attr($item_id); ?>">
								<?php esc_html_e( 'CSS Classes (optional)', 'goodlayers-core' ); ?><br />
								<input type="text" id="edit-menu-item-classes-<?php echo esc_attr($item_id); ?>" class="widefat code edit-menu-item-classes" name="menu-item-classes[<?php echo esc_attr($item_id); ?>]" value="<?php echo esc_attr( implode(' ', $item->classes ) ); ?>" />
							</label>
						</p>
						<p class="field-xfn description description-thin">
							<label for="edit-menu-item-xfn-<?php echo esc_attr($item_id); ?>">
								<?php esc_html_e( 'Link Relationship (XFN)', 'goodlayers-core' ); ?><br />
								<input type="text" id="edit-menu-item-xfn-<?php echo esc_attr($item_id); ?>" class="widefat code edit-menu-item-xfn" name="menu-item-xfn[<?php echo esc_attr($item_id); ?>]" value="<?php echo esc_attr( $item->xfn ); ?>" />
							</label>
						</p>
						<p class="field-description description description-wide">
							<label for="edit-menu-item-description-<?php echo esc_attr($item_id); ?>">
								<?php esc_html_e( 'Description', 'goodlayers-core' ); ?><br />
								<textarea id="edit-menu-item-description-<?php echo esc_attr($item_id); ?>" class="widefat edit-menu-item-description" rows="3" cols="20" name="menu-item-description[<?php echo esc_attr($item_id); ?>]"><?php echo esc_html( $item->description ); // textarea_escaped ?></textarea>
								<span class="description"><?php esc_html_e('The description will be displayed in the menu if the current theme supports it.', 'goodlayers-core'); ?></span>
							</label>
						</p>
						
						<?php do_action('gdlr_core_walker_nav_menu_edit_fields', $item); ?>
						
						<p class="field-move hide-if-no-js description description-wide">
							<label>
								<span><?php esc_html_e( 'Move', 'goodlayers-core' ); ?></span>
								<a href="#" class="menus-move menus-move-up" data-dir="up"><?php esc_html_e( 'Up one', 'goodlayers-core' ); ?></a>
								<a href="#" class="menus-move menus-move-down" data-dir="down"><?php esc_html_e( 'Down one', 'goodlayers-core' ); ?></a>
								<a href="#" class="menus-move menus-move-left" data-dir="left"></a>
								<a href="#" class="menus-move menus-move-right" data-dir="right"></a>
								<a href="#" class="menus-move menus-move-top" data-dir="top"><?php esc_html_e( 'To the top', 'goodlayers-core' ); ?></a>
							</label>
						</p>

						<div class="menu-item-actions description-wide submitbox">
							<?php if ( 'custom' != $item->type && $original_title !== false ) : ?>
								<p class="link-to-original">
									<?php printf( esc_html__('Original: %s', 'goodlayers-core'), '<a href="' . esc_attr( $item->url ) . '">' . esc_html( $original_title ) . '</a>' ); ?>
								</p>
							<?php endif; ?>
							<a class="item-delete submitdelete deletion" id="delete-<?php echo esc_attr($item_id); ?>" href="<?php
							echo wp_nonce_url(
								add_query_arg(
									array(
										'action' => 'delete-menu-item',
										'menu-item' => $item_id,
									),
									admin_url( 'nav-menus.php' )
								),
								'delete-menu_item_' . $item_id
							); ?>"><?php esc_html_e( 'Remove', 'goodlayers-core' ); ?></a> <span class="meta-sep hide-if-no-js"> | </span> <a class="item-cancel submitcancel hide-if-no-js" id="cancel-<?php echo esc_attr($item_id); ?>" href="<?php echo esc_url( add_query_arg( array( 'edit-menu-item' => $item_id, 'cancel' => time() ), admin_url( 'nav-menus.php' ) ) );
								?>#menu-item-settings-<?php echo esc_attr($item_id); ?>"><?php esc_html_e('Cancel', 'goodlayers-core'); ?></a>
						</div>

						<input class="menu-item-data-db-id" type="hidden" name="menu-item-db-id[<?php echo esc_attr($item_id); ?>]" value="<?php echo esc_attr($item_id); ?>" />
						<input class="menu-item-data-object-id" type="hidden" name="menu-item-object-id[<?php echo esc_attr($item_id); ?>]" value="<?php echo esc_attr( $item->object_id ); ?>" />
						<input class="menu-item-data-object" type="hidden" name="menu-item-object[<?php echo esc_attr($item_id); ?>]" value="<?php echo esc_attr( $item->object ); ?>" />
						<input class="menu-item-data-parent-id" type="hidden" name="menu-item-parent-id[<?php echo esc_attr($item_id); ?>]" value="<?php echo esc_attr( $item->menu_item_parent ); ?>" />
						<input class="menu-item-data-position" type="hidden" name="menu-item-position[<?php echo esc_attr($item_id); ?>]" value="<?php echo esc_attr( $item->menu_order ); ?>" />
						<input class="menu-item-data-type" type="hidden" name="menu-item-type[<?php echo esc_attr($item_id); ?>]" value="<?php echo esc_attr( $item->type ); ?>" />
						<div class="clear" ></div>
					</div><!-- .menu-item-settings-->
					<ul class="menu-item-transport"></ul>
				<?php
				$output .= ob_get_clean();
			}
			
		} // gdlr_core_edit_nav_menu_walker
	} // class_exists