<?php
	/*	
	*	Goodlayers Page Builder
	*	---------------------------------------------------------------------
	*	File which creates page builder elements
	*	---------------------------------------------------------------------
	*/
	
	if( !class_exists('gdlr_core_page_builder') ){
		
		class gdlr_core_page_builder{

			// static function for section registration
			private static $pb;
			private static $pb_section = array();
			
			static function get(){
				return self::$pb;
			}
			static function add_section( $slug, $class_name ){
				self::$pb_section[$slug] = $class_name;
			}
			static function get_section( $slug ){
				return self::$pb_section[$slug];
			}
			
			// creating object
			private $settings = array();
			
			function __construct( $settings = array() ){
				
				$this->settings = wp_parse_args($settings, array(
					'post_type' => array('page'),
					'script' => array(),
					'style' => array(),
					'slug' => 'gdlr-core-page-builder',
					'view_mode' => get_option('gdlr-core-page-builder-view-mode', 'live')
				));

				// additional style
				$this->settings['style'] = apply_filters('gdlr_core_page_builder_additional_style', $this->settings['style']);
				
				// create custom meta box
				add_action('add_meta_boxes', array(&$this, 'init_page_builder_meta_box'), 9);
				
				// save custom metabox
				$this->settings['post_type'] = apply_filters('gdlr_core_page_builder_post_type', $this->settings['post_type']);
				foreach( $this->settings['post_type'] as $post_type ){
					add_action('save_post_' . $post_type , array(&$this, 'save_page_builder_meta_box'));
				}
				
				// print page builder item
				add_action('gdlr_core_print_page_builder', array(&$this, 'print_page_builder'));

				// include search for page builder
				// add_filter('posts_search', array(&$this, 'search_page_builder'), 10, 2);
				// search page builder
                add_filter('posts_join', array(&$this, 'search_posts_join'), 10, 2);
                add_filter('posts_where', array(&$this, 'search_posts_where'), 10, 2);
                add_filter('posts_groupby', array(&$this, 'search_posts_groupby'), 10, 2);

				// ajax action
				add_action('wp_ajax_gdlr_core_get_pb_options', array(&$this, 'get_item_options'));
				add_action('wp_ajax_gdlr_core_get_skin_options', array(&$this, 'get_skin_options'));
				add_action('wp_ajax_gdlr_core_update_skin', array(&$this, 'update_skin'));
				add_action('wp_ajax_gdlr_core_save_page_builder_data', array(&$this, 'save_page_builder_data'));
				add_action('wp_ajax_gdlr_core_pb_update_option', array(&$this, 'pb_update_option'));

				// add the script when opening the registered post type
				add_action('admin_enqueue_scripts', array(&$this, 'load_page_builder_script') );
				
				// add page builder meta to revision
				if( class_exists('gdlr_core_revision') ){
					gdlr_core_revision::add_field(array(
						'meta_key'=>$this->settings['slug'], 
						'meta_name'=>esc_html__('Page Builder', 'goodlayers-core'),
						'callback'=>array(&$this, 'convert_page_builder_revision_data')
					));
				}
				
				// init the class instace if exists
				foreach( self::$pb_section as $section_class ){
					call_user_func($section_class . '::init');
				}

				// init static variable
				self::$pb = $this;
			}
			
			// function that enqueue page builder script
			function load_page_builder_script( $hook ){
				if( $hook == 'template-settings' || (($hook == 'post.php' || $hook == 'post-new.php') && in_array(get_post_type(), $this->settings['post_type'])) ){
					
					// include the script for popup options
					gdlr_core_html_option::include_script(array(
						'style' => 'html-option-small'
					));					
					
					// include the page builder necessary style
					gdlr_core_include_icon_font('font-awesome');
					wp_enqueue_style('gdlr-core-page-builder-style', GDLR_CORE_URL . '/framework/css/page-builder.css', array(), '1.7.0');
				
					// include the page builder necessary script
					wp_enqueue_script('jquery-ui-core');
					wp_enqueue_script('jquery-ui-draggable');
					wp_enqueue_script('jquery-ui-sortable');
					wp_enqueue_script('jquery-ui-droppable');
					
					wp_enqueue_script('gdlr-core-page-builder-script', GDLR_CORE_URL . '/framework/js/page-builder.js', array('jquery'), '1.7.6', true);
				
					// send the variable to page builder script
					$page_builder_var = array();
					$page_builder_var['post_id'] = get_the_ID();
					$page_builder_var['settings'] = array(
						'undo_times' => 20
					);
					$page_builder_var['text'] = array(
						'item_added' => esc_html__('Item Added!', 'goodlayers-core'),
						'ajaxurl' => GDLR_CORE_AJAX_URL,
						'error_head' => esc_html__('An error occurs', 'goodlayers-core'),
						'error_message' => esc_html__('Please refresh the page to try again. If the problem still persists, please contact administrator for this.', 'goodlayers-core'),
						'undo_end' => esc_html__('Unable to undo changes', 'goodlayers-core'),
						'redo_end' => esc_html__('Unable to redo changes', 'goodlayers-core'),
					);
					
					// get template for creating elements
					$page_builder_var['template'] = array();
					foreach( self::$pb_section as $section_slug => $section_class ){
						$page_builder_var = call_user_func( $section_class . '::set_page_builder_var', $page_builder_var );
					}
					
					wp_localize_script('gdlr-core-page-builder-script', 'page_builder_var', $page_builder_var);
					
					////////////////////////////////////////////
					// front end script
					////////////////////////////////////////////
					
					// custom style for visual on page builder
					wp_deregister_script('jquery-transit');

					// include frontend script
					gdlr_core_front_script(true);
					
					// additional style ( eg. color from theme )
					foreach( $this->settings['style'] as $slug => $style ){
						wp_enqueue_style($slug, $style);
					}
					
					// google font query
					global $gdlr_core_font_loader;
					if( empty($gdlr_core_font_loader) ){
						$gdlr_core_font_loader = new gdlr_core_font_loader();
					}
					$gdlr_core_font_loader->google_font_enqueue();

					if( is_rtl() ){
						wp_enqueue_style('frontend-rtl', GDLR_CORE_URL . '/include/css/backend-rtl.css');
					}

					do_action('gdlr_core_load_page_builder_script');
				}
			}
			
			// function that creats page builder meta box
			function init_page_builder_meta_box(){
				foreach( $this->settings['post_type'] as $post_type ){
					add_meta_box( 'gdlr-core-page-builder-meta', esc_html__('Page Builder', 'goodlayers-core'),
						array(&$this, 'create_page_builder_meta_box'),
						$post_type, 'normal', 'high' );	
				}
			}
			
			/////////////////////
			// content area
			/////////////////////
			
			function create_page_builder_meta_box( $post = '', $args = array() ){

				// add nonce field to validate upon saving
				wp_nonce_field('gdlr_core_page_builder', 'page_builder_security');
				
				global $pagenow;
				$page_builder_val = '';
				if( $pagenow == 'post-new.php' ){
					$page_builder_val = apply_filters('gdlr_core_' . get_post_type() . '_page_builder_val_init', array());
				}else if( !empty($post) ){
					$page_builder_val = get_post_meta($post->ID, $this->settings['slug'], true);
				}else if( !empty($args['value']) ){
					$page_builder_val = $args['value'];
				}
				$page_builder_val = apply_filters('gdlr_core_page_builder_val', $page_builder_val);

				echo '<input type="hidden" id="gdlr-core-page-builder-val" name="' . esc_attr($this->settings['slug']) . '" value="' . esc_attr(json_encode($page_builder_val)) . '"/>';
				
				$this->get_pb_tooltip();

				echo '<div class="gdlr-core-page-builder" id="gdlr-core-page-builder" >';
				
				// page builder head area
				$section_active = 'wrapper';
				echo '<div class="gdlr-core-page-builder-head-sub"></div>';
				echo '<div class="gdlr-core-page-builder-head">';
				echo '<div class="gdlr-core-page-builder-head-nav" id="gdlr-core-page-builder-head-nav">';
				foreach( self::$pb_section as $section_slug => $section_class ){
					if( !empty($args['display-section']) && !in_array($section_slug, $args['display-section']) ){
						continue;
					}
					
					$settings = call_user_func( $section_class . '::get_settings' );
					echo '<div class="gdlr-core-page-builder-head-nav-item ' . ($section_slug == $section_active? 'gdlr-core-active': '') . '" data-nav-type="' . esc_attr($section_slug) . '" >';
					if( !empty($settings['icon']) ){
						echo '<img src="' . esc_url($settings['icon']) . '" />';
					}
					echo gdlr_core_escape_content($settings['title']);
					echo '</div>';
				}
				
				echo '<div class="gdlr-core-page-builder-head-full-screen" id="gdlr-core-page-builder-head-full-screen"  ></div>';

				echo '<div class="gdlr-core-page-builder-head-nav-update" id="gdlr-core-page-builder-head-nav-update" data-post-id="' . esc_attr(get_the_ID()) . '" >';
				echo '<i class="fa fa-save"></i>' . esc_html__('Save PB', 'goodlayers-core');
				echo '</div>';

				echo '<div class="gdlr-core-page-builder-head-action-live-mode-alt" id="gdlr-core-page-builder-head-action-live-mode-alt" data-mode="' . esc_attr($this->settings['view_mode']) . '" >';
				echo '<span class="gdlr-core-page-builder-head-action-live-mode-button gdlr-core-active" data-live-mode="edit" >' . esc_html__('Edit', 'goodlayers-core') . '</span>';
				echo '<span class="gdlr-core-page-builder-head-action-live-mode-button" data-live-mode="preview" >' . esc_html__('Preview', 'goodlayers-core') . '</span>';
				echo '</div>';

				echo '<div class="gdlr-core-page-builder-head-action-mode-section-alt" id="gdlr-core-page-builder-head-action-mode-section" data-mode="' . esc_attr($this->settings['view_mode']) . '" >';
				echo '<div class="gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-block" data-mode="block" >';
				echo '<i class="fa fa-th-large" ></i><span class="gdlr-core-head">' . esc_html__('Block Mode', 'goodlayers-core') . '</span>';
				echo '</div>';
				echo '<div class="gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-preview" data-mode="preview" >';
				echo '<i class="fa fa-eye" ></i><span class="gdlr-core-head">' . esc_html__('Preview Mode', 'goodlayers-core') . '</span>';
				echo '</div>';
				
				echo '<div class="gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-live" data-mode="live" >';
				echo '<i class="fa fa-tv" ></i><span class="gdlr-core-head">' . esc_html__('Live Mode', 'goodlayers-core') . '</span>';
				echo '</div>';
				echo '</div>'; // gdlr-core-page-builder-head-action-mode-section

				// undo redo
				echo '<div class="gdlr-core-page-builder-head-action-repeat-alt" id="gdlr-core-page-builder-head-action-redo-alt" >';
				echo '<i class="fa fa-repeat" ></i>';
				echo '</div>';
				echo '<div class="gdlr-core-page-builder-head-action-repeat-alt" id="gdlr-core-page-builder-head-action-undo-alt" >';
				echo '<i class="fa fa-undo" ></i>';
				echo '</div>';

				echo '<div class="clear"></div>';
				echo '</div>';
				
				echo '<div class="gdlr-core-page-builder-head-content" id="gdlr-core-page-builder-head-content" >';
				foreach( self::$pb_section as $section_slug => $section_class ){
					if( !empty($args['display-section']) && !in_array($section_slug, $args['display-section']) ){
						continue;
					}

					$content_class  = 'gdlr-core-page-builder-head-content-' . $section_slug;
					$content_class .= ($section_slug == $section_active)? ' gdlr-core-active': '';

					echo '<div class="' . esc_attr($content_class) . '" data-content-type="' . esc_attr($section_slug) . '" >';
					call_user_func($section_class . '::get_element_list', $args);
					echo '</div>';
				}
				echo '</div>';	
				
				$this->get_head_action_bar($args);
				echo '</div>'; // gdlr-core-page-builder-head
				
				// page builder container area
				$view_mode_class  = ($this->settings['view_mode'] == 'live')? 'gdlr-core-pb-livemode': 'gdlr-core-pb-blockmode';
				$view_mode_class .= ' gdlr-core-live-edit';
				$page_builder_body_class = apply_filters('gdlr_core_page_builder_body_class', '');

				echo '<div class="gdlr-core-page-builder-body gdlr-core-no-dragging gdlr-core-body ' . esc_attr($view_mode_class) . ' ' . esc_attr($page_builder_body_class) . '" id="gdlr-core-page-builder-body" data-mode="' . esc_attr($this->settings['view_mode']) . '" >';
				echo '<div class="gdlr-core-page-builder-container gdlr-core-container gdlr-container-sortable clearfix" id="gdlr-core-page-builder-container" >';
				echo gdlr_core_escape_content(gdlr_core_page_builder::get_page_builder_item($page_builder_val));
				echo '</div>'; // gdlr-core-page-builder-container
				echo '</div>'; // gdlr-core-page-builder-body
				
				echo '</div>'; // gdlr-core-page-builder
			}
			
			// print saved item
			static function get_page_builder_item( $page_builder_val ){
				
				$ret = '';
				
				if( !empty($page_builder_val) ){
					foreach( $page_builder_val as $page_builder_option ){
						if( empty($page_builder_option['template']) || $page_builder_option['template'] == 'template' ) continue;
						
						$ret .= call_user_func(	
							self::$pb_section[$page_builder_option['template']] . '::get_template', 
							$page_builder_option, 
							'gdlr_core_page_builder::get_page_builder_item'
						);
					}
				}
				
				return $ret;
			}
			
			// print the action bar on page builder head area
			function get_head_action_bar($args = array()){
				
				echo '<div class="gdlr-core-page-builder-head-action-bar" >';
				
				// reset
				echo '<div class="gdlr-core-page-builder-head-action-bar-left" >';
				echo '<div class="gdlr-core-page-builder-head-action-button" id="gdlr-core-page-builder-head-action-reset" >';
				echo '<i class="fa fa-remove" ></i>' . esc_html__('Reset', 'goodlayers-core');
				echo '</div>';
				
				// save
				if( empty($args['exclude-support']) || !in_array('save-template', $args['exclude-support']) ){
					echo '<div class="gdlr-core-page-builder-head-action-button" id="gdlr-core-page-builder-head-action-save-template" >';
					echo '<i class="fa fa-download" ></i>' . esc_html__('Save Template', 'goodlayers-core');
					echo '</div>';
					echo '<div class="clear"></div>';
				}
				echo '</div>'; // gdlr-core-page-builder-head-action-bar-left
				
				// view mode
				echo '<div class="gdlr-core-page-builder-head-action-bar-right" >';
				
				echo '<div class="gdlr-core-page-builder-head-action-mode-section" id="gdlr-core-page-builder-head-action-mode-section" data-mode="' . esc_attr($this->settings['view_mode']) . '" >';
				echo '<div class="gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-block" data-mode="block" >';
				echo '<i class="fa fa-th-large" ></i><span class="gdlr-core-head">' . esc_html__('Block Mode', 'goodlayers-core') . '</span>';
				echo '</div>';
				echo '<div class="gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-preview" data-mode="preview" >';
				echo '<i class="fa fa-eye" ></i><span class="gdlr-core-head">' . esc_html__('Preview Mode', 'goodlayers-core') . '</span>';
				echo '</div>';
				
				echo '<div class="gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-live" data-mode="live" >';
				echo '<i class="fa fa-tv" ></i><span class="gdlr-core-head">' . esc_html__('Live Mode', 'goodlayers-core') . '</span>';
				echo '</div>';
				echo '</div>'; // gdlr-core-page-builder-head-action-mode-section
				
				// live mode option
				echo '<div class="gdlr-core-page-builder-head-action-live-mode" id="gdlr-core-page-builder-head-action-live-mode" >';
				echo '<span class="gdlr-core-page-builder-head-action-live-mode-button gdlr-core-active" data-live-mode="edit" >' . esc_html__('Edit', 'goodlayers-core') . '</span>';
				echo '<span class="gdlr-core-page-builder-head-action-live-mode-button" data-live-mode="preview" >' . esc_html__('Preview', 'goodlayers-core') . '</span>';
				echo '</div>';
				
				echo '<div class="gdlr-core-page-builder-head-action-repeat" id="gdlr-core-page-builder-head-action-undo" >';
				echo '<i class="fa fa-undo" ></i>';
				echo '</div>';
				echo '<div class="gdlr-core-page-builder-head-action-repeat" id="gdlr-core-page-builder-head-action-redo" >';
				echo '<i class="fa fa-repeat" ></i>';
				echo '</div>';
				echo '<div class="clear"></div>';
				echo '</div>'; // gdlr-core-page-builder-head-action-bar-right
				
				echo '<div class="clear"></div>';
				echo '</div>'; // gdlr-core-page-builder-head-action-bar
				
			} // get_head_action_bar

			function get_pb_tooltip(){

				$page_builder_tip = get_option('gdlr-core-page-builder-tip', '0');
				if( empty($page_builder_tip) ){
					echo '<div class="gdlr-core-page-builder-tip" id="gdlr-core-page-builder-tip" >';
					echo '<div class="gdlr-core-page-builder-tip-head" >' . esc_html__('Tip', 'goodlayers-core') . '</div>';
					echo '<div class="gdlr-core-page-builder-tip-content" >' . esc_html__('Do you know you can also use double click to edit the item in live edit/preview mode ?', 'goodlayers-core') . '</div>';
					echo '<div class="gdlr-core-page-builder-tip-dismiss" >' . esc_html__('Dismiss', 'goodlayers-core') . '</div>';
					echo '</div>';
				}

			} // get_pb_tooltip
			
			/////////////////////
			// ajax section
			/////////////////////		
			
			function get_item_options(){
				
				if( !check_ajax_referer('gdlr_core_page_builder', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}
				
				if( empty($_POST['template']) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('An Error Occurs', 'goodlayers-core'),
						'message'=> esc_html__('Could not locate an option\'s template.' ,'goodlayers-core')
					)));
				}else{
					$_POST['value'] = empty($_POST['value'])? array(): gdlr_core_process_post_data($_POST['value']);
					$item_options = call_user_func(self::$pb_section[$_POST['template']] . '::get_options');
					$pb_options = new gdlr_core_page_builder_options($item_options, $_POST['value']);
					$content = $pb_options->get_content();
				}
				
				die(json_encode(array(
					'status' => 'success',
					'option_content' => $content
				))); 
			} // get_item_options
			
			function get_skin_options(){

				if( !check_ajax_referer('gdlr_core_page_builder', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}
				
				// get skin to show
				$content  = '<div class="gdlr-core-page-builder-options gdlr-core-page-builder-skin-options" >';
				$content .= '<div class="gdlr-core-page-builder-options-head">';
				$content .= '<div class="gdlr-core-page-builder-options-head-title">';
				$content .= '<i class="fa fa-gears"></i>';
				$content .= esc_html__('Skin Options', 'goodlayers-core');
				$content .= '</div>'; // gdlr-core-page-builder-options-head-title
				$content .= '<span class="gdlr-core-page-builder-options-head-close" id="gdlr-core-page-builder-options-head-close" ></span>';
				$content .= '</div>'; // gdlr-core-page-builder-options-head
				
				$content .= '<div class="gdlr-core-page-builder-options-content">';
				$content .= '<div class="gdlr-core-page-builder-options-tab-content" >';
				$content .= '<div class="gdlr-core-page-builder-option-tab-content-item gdlr-core-active" >';
				$content .= gdlr_core_html_option::get_element(array(
					'slug' => 'skin',
					'value' => gdlr_core_skin_settings::get_option(),
					'title' => esc_html__('Skin Settings', 'infinite'),
					'type' => 'custom',
					'item-type' => 'skin-settings',
					'wrapper-class' => 'gdlr-core-fullsize',
					'options' => apply_filters('gdlr_core_skin_options', array())
				));
				$content .= '</div>';
				$content .= '</div>';
				$content .= '</div>'; 
				
				$content .= '<div class="gdlr-core-page-builder-options-end">';
				$content .= '<div class="gdlr-core-page-builder-options-save" id="gdlr-core-page-builder-options-save" >';
				$content .= '<i class="fa fa-save"></i>' . esc_html__('Save Options', 'goodlayers-core');
				$content .= '</div>';
				$content .= '</div>';
				$content .= '</div>';
				
				die( json_encode(array(
					'status' => 'success',
					'option_content' => $content
				)) ); 

			} // get_skin_options

			function update_skin(){

				if( !check_ajax_referer('gdlr_core_page_builder', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce ( Cannot update skin )', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}

				if( !empty($_POST['value']['skin']) ){
					$ret = array( 'status' => 'success' );

					$skin_val = gdlr_core_process_post_data($_POST['value']['skin']);
					gdlr_core_skin_settings::update_option($skin_val);

					$ret['style'] = gdlr_core_skin_settings::get_skins_style();

					$ret['combobox'] = '';
					$skin_list = gdlr_core_skin_settings::get_skins();
					$prev_value = empty($_POST['skin'])? '': gdlr_core_process_post_data($_POST['skin']);
					foreach( $skin_list as $skin_slug => $skin_name ){
						$ret['combobox'] .= '<option value="' . esc_attr($skin_slug) . '" ' . selected($prev_value, $skin_slug, false) . ' >' . gdlr_core_escape_content($skin_name) . '</option>';
					}

					die(json_encode($ret));

				}else{
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Empty Input Data', 'goodlayers-core'),
						'message'=> esc_html__('Please try updating the skin again.' ,'goodlayers-core')
					)));
				}

			} // update skin

			/////////////////////
			// save section
			/////////////////////
			
			function save_page_builder_meta_box( $post_id ){
				
				// check if nonce is available
				if( !isset($_POST['page_builder_security']) ){
					return;
				}

				// vertify that the nonce is vaild
				if( !wp_verify_nonce($_POST['page_builder_security'], 'gdlr_core_page_builder') ) {
					return;
				}

				// ignore the auto save
				if( defined( 'DOING_AUTOSAVE' ) && DOING_AUTOSAVE ){
					return;
				}

				// check the user's permissions.
				if( isset($_POST['post_type']) && 'page' == $_POST['post_type'] ) {
					if( !current_user_can('edit_page', $post_id) ){
						return;
					}
				}else{
					if( !current_user_can('edit_post', $post_id) ){
						return;
					}
				}			
				
				// start updating the meta fields
				$value = json_decode(gdlr_core_process_post_data($_POST[$this->settings['slug']]), true);
				$value = gdlr_core_sanitize_input($value);
				
				if( is_array($value) ){
					update_post_meta($post_id, $this->settings['slug'], $value);
					gdlr_core_page_builder_sync_template::update_sync_template($value);
				}

				// update the tool tip text
				if( !empty($_POST['gdlr-core-page-builder-tip']) ){
					update_option('gdlr-core-page-builder-tip', 1);
				}

				delete_post_meta($post_id, 'gdlr-core-pb-inline-css');
			}
			function save_page_builder_data(){
				if( !check_ajax_referer('gdlr_core_page_builder', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please use wordpress update button to update the page instead.' ,'goodlayers-core')
					)));
				}
				
				if( !empty($_POST['post_id']) ){

					$value = json_decode(gdlr_core_process_post_data($_POST['value']), true);
					$value = gdlr_core_sanitize_input($value);
					if( !empty($value) ){
						update_post_meta($_POST['post_id'], $this->settings['slug'], $value);
						gdlr_core_page_builder_sync_template::update_sync_template($value);
					}

					delete_post_meta($_POST['post_id'], 'gdlr-core-pb-inline-css');

					die(json_encode(array(
						'status' => 'success',
						'head' => esc_html__('Successfully Save', 'goodlayers-core'),
						'message'=> ''
					)));

				}else if( !empty($_POST['template_type']) ){

					$template_type = $_POST['template_type'];
					$template_id = intval($_POST['template_id']);
					$template_title = gdlr_core_process_post_data($_POST['template_title']);
					$value = json_decode(gdlr_core_process_post_data($_POST['value']), true);
					$value = gdlr_core_sanitize_input($value);
					$option_slug = 'gdlr-core-' . $template_type . '-template-setting';

					$templates = get_option($option_slug, array());
					$templates[$template_id] = array(
						'title' => $template_title,
						'value' => $value
					);
					update_option($option_slug, $templates);

					die(json_encode(array(
						'status' => 'success',
						'head' => esc_html__('Successfully Save', 'goodlayers-core'),
						'message'=> ''
					)));

				}else{
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Post Data', 'goodlayers-core'),
						'message'=> esc_html__('Please use wordpress update button to update the page instead.' ,'goodlayers-core')
					)));
				}
			}
			function pb_update_option(){

				if( !check_ajax_referer('gdlr_core_page_builder', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce ( Cannot update skin )', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}

				if( isset($_POST['slug']) && !in_array($_POST['slug'], array('gdlr-core-page-builder-tip', 'gdlr-core-page-builder-view-mode')) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Permission Denied', 'goodlayers-core'),
						'message'=> esc_html__('Please contact administrator.' ,'goodlayers-core')
					)));
				}

				if( isset($_POST['slug']) && isset($_POST['value']) ){
					$slug = gdlr_core_process_post_data($_POST['slug']);
					$value = gdlr_core_process_post_data($_POST['value']);
					update_option($slug, $value);
					die(json_encode(array(
						'status' => 'success'
					)));
				}else{
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Empty Input Data', 'goodlayers-core'),
						'message'=> esc_html__('Something went wrong, please try contacting the administrator.' ,'goodlayers-core')
					)));
				}

			}
			
			// convert the data to read able revision format
			function convert_page_builder_revision_data( $data ){
				return json_encode($data) . "\n";
			}

			//////////////////////////////////////
			// display page builder on front end
			//////////////////////////////////////
			function print_page_builder( $post_id = '' ){
				$post_id = empty($post_id)? get_the_ID(): $post_id;

				$page_builder_val = get_post_meta($post_id, $this->settings['slug'], true);
				$page_builder_val = apply_filters('gdlr_core_page_builder_val', $page_builder_val);
				$page_builder_body_class = apply_filters('gdlr_core_page_builder_body_class', '');

				if( !empty($page_builder_val) ){
					echo '<div class="gdlr-core-page-builder-body clearfix">' . $this->print_page_builder_item( $page_builder_val, true ) . '</div>';
				}
			}		
			function print_page_builder_item( $page_builder_val, $outer_section = false ){
				
				$ret = '';
				$column_count = 0;
				$element_count = 0;
				
				if( !empty($page_builder_val) ){
					foreach( $page_builder_val as $page_builder_option ){
						if( empty($page_builder_option) ) continue;

						if( $page_builder_option['template'] == 'wrapper' && !in_array($page_builder_option['type'], array('column', 'flexbox')) ){
							
							// close the section if previous element isn't wrapper item 
							if( $outer_section && ($column_count > 0 || $element_count > 0) ){
								$column_count = 0;
								$element_count = 0;
								
								$ret .= '</div>'; // gdlr-core-pbf-section-container
								$ret .= '</div>'; // gdlr-core-pbf-section
							}

						}else{

							// open section if first element
							if( $outer_section && $column_count == 0 && $element_count == 0 ){
								$ret .= '<div class="gdlr-core-pbf-section" >';
								$ret .= '<div class="gdlr-core-pbf-section-container gdlr-core-container clearfix" >';
							}

							if( $page_builder_option['type'] == 'column' ){
								$page_builder_option['column'] = empty($page_builder_option['column'])? 0: $page_builder_option['column'];
								
								if( $column_count == 0 ){
									$page_builder_option['first-column'] = true;
									$column_count += intval($page_builder_option['column']);
								}else{
									$column_count += intval($page_builder_option['column']);
									if( $column_count > 60 ){
										$page_builder_option['first-column'] = true;
										$column_count = intval($page_builder_option['column']);
									}
								}
							}else{
								$element_count++;
							}						
						}
						
						if( in_array($page_builder_option['template'], array('wrapper', 'element'))){
							$ret .= call_user_func(	
								self::$pb_section[$page_builder_option['template']] . '::get_content', 
								$page_builder_option, 
								array(&$this, 'print_page_builder_item')
							);
						}
						
					}

					// close the section if previous element isn't wrapper item 
					if( $outer_section && ($column_count > 0 || $element_count > 0) ){
						$column_count = 0;
						$element_count = 0;
						
						$ret .= '</div>'; // gdlr-core-pbf-section-container
						$ret .= '</div>'; // gdlr-core-pbf-section
					}
				}
				return $ret;
			}


			// search page builder
			// ref: https://gist.github.com/charleslouis/5924863
			function search_posts_join($join, $query){
                global $wpdb;
                
                $keywords = $query->get('s');
                if( !empty($keywords) && strpos($join, $wpdb->postmeta) === false ){
                    $join .= " LEFT JOIN {$wpdb->postmeta} ON {$wpdb->posts}.ID = {$wpdb->postmeta}.post_id ";
                }

                return $join;
            }
            function search_posts_where($where, $query) {
                global $wpdb;
                
                // Search post_meta
                $keywords = $query->get('s');
                if( !empty($keywords) ){
					
					// ignore attachment search
                    if( strpos($where, ".post_type = 'attachment'") !== false ){
						return $where;
					}

					// search v3
					$meta_key = $wpdb->prepare("{$wpdb->postmeta}.meta_key = %s", $this->settings['slug']);
					$where = preg_replace(
						"/\(\s*{$wpdb->posts}\.post_title\s+LIKE\s*(\'[^\']+\')\s*\)/",
						"({$wpdb->posts}.post_title LIKE $1) OR ({$meta_key} AND {$wpdb->postmeta}.meta_value LIKE $1)",
						$where
					);
                }

                return $where;
            }
            function search_posts_groupby($groupby, $query){
                global $wpdb;

                // Ensures no duplicate results
                $keywords = $query->get('s');
                if( !empty($keywords) ){
                    return "{$wpdb->posts}.ID";
                }

                return $groupby;
            }

			// old search
			function search_page_builder( $where, $wp_query ){

				if( empty( $where )) return $where;
			 	
				global $wpdb;
				
				$consition_pos_s = strpos($where, "({$wpdb->prefix}posts.post_title LIKE") + 1;
				$consition_pos_e = strpos($where, ")", $consition_pos_s);
				$keyword = substr($where, $consition_pos_s, $consition_pos_e - $consition_pos_s);
				$keyword = str_replace("{$wpdb->prefix}posts.post_title", "meta_value", $keyword);
				
				$pb_search  = "SELECT post_id FROM {$wpdb->prefix}postmeta ";
				$pb_search .= "WHERE meta_key = '{$this->settings['slug']}' AND {$keyword} ";
				
				$where = substr($where, 0, $consition_pos_s - 1) . " {$wpdb->prefix}posts.ID IN({$pb_search}) OR " . substr($where, $consition_pos_s - 1);
				// echo $keyword . ':::<br>';
				return $where;

				/*
				// reset search in order to rebuilt it as we whish
   				$where = '';

			    // get search expression
    			$terms = $wp_query->query_vars['s'];

				// explode search expression to get search terms
				$exploded = explode( ' ', $terms );
				if( $exploded === FALSE || count($exploded) == 0 ){
				   $exploded = array( 0 => $terms );
				}

				// search acf
				$search_slug = $this->settings['slug'];

				foreach( $exploded as $tag ){
					$tag = esc_sql($tag);

			        $where .= " 
				        AND (
				        	({$wpdb->prefix}posts.post_title LIKE '%$tag%')
				            OR ({$wpdb->prefix}posts.post_excerpt LIKE '%$tag%')
				            OR ({$wpdb->prefix}posts.post_content LIKE '%$tag%')
				            OR EXISTS (
				              SELECT * FROM {$wpdb->prefix}postmeta
					              WHERE post_id = {$wpdb->prefix}posts.ID
					                AND (meta_key = '" . $search_slug . "' AND meta_value LIKE '%{$tag}%')
			            	)
			        	)
			        ";
			    }

				return $where;
				*/
			}
			
		} // gdlr_core_page_builder
		
	} // class_exists
	
	if( !class_exists('gdlr_core_page_builder_section') ){
		class gdlr_core_page_builder_section{
			
			// init the action/variable 
			static function init(){
			}
			
			// getting the element setttings -> title / icon 
			static function get_settings(){
				return array();
			}
			
			// assign the page builder variable
			static function set_page_builder_var( $page_builder_var ){
				return $page_builder_var;
			}
			
			// getting the list of elements to page builder bar
			static function get_element_list(){
			}
			
			// the template that sent over the localize function
			static function get_template( $options = array(), $callback = '' ){ 
				return ''; 
			}		
			
			// get the option for displaying
			// static function get_options();
		}
	}