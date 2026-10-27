<?php
	/*	
	*	Goodlayers Sidebar Generator
	*	---------------------------------------------------------------------
	*	This file create the class that help you to controls the sidebar 
	*	at the appearance > widget area
	*	---------------------------------------------------------------------
	*/
	
	if( !class_exists('gdlr_core_sidebar_generator') ){
		
		class gdlr_core_sidebar_generator{
			
			static function get_sidebars(){
				global $wp_registered_sidebars;
				
				$sidebars = array();
				if( !empty($wp_registered_sidebars) && is_array($wp_registered_sidebars) ){
					foreach( $wp_registered_sidebars as $sidebar_id => $value ) {
						$sidebars[$sidebar_id] = $value['name'];
					}
				}
				
				return $sidebars;
			}
			
			private $sidebar_option_name = 'gdlrcst_sidebar_name';
			private $sidebars = array();
			private $footer_widgets = array();
			private $sidebar_args = array();
			
			function __construct( $sidebar_args = array() ){
				
				// initialize the variable
				$footer_widget_amount = apply_filters('gdlr_core_footer_widget_amount', 4);
				$this->footer_widgets = array();
				for( $i = 1; $i <= $footer_widget_amount; $i++ ){
					$this->footer_widgets[] = array( 
						'name' => sprintf('Footer %d', $i), 
						'id' => 'footer-' . $i, 
						'description' => sprintf('Footer Column %d', $i)
					);
				}
				
				$this->sidebars = get_option($this->sidebar_option_name, array());
				if( !is_array($this->sidebars) ){ $this->sidebars = array(); }
				
				$this->sidebar_args = wp_parse_args( $sidebar_args, array(
					'before_widget' => '<div id="%1$s" class="widget %2$s gdlr-core-widget">',
					'after_widget'  => '</div>',
					'before_title'  => '<h3 class="gdlr-core-widget-title">',
					'after_title'   => '</h3><div class="clear"></div>' ) );
				
				// add action to register existing sidebar
				add_action('widgets_init', array(&$this, 'register_sidebar_widget'));
				
				global $wp_customize;
				
				if( !isset($wp_customize) ){
				
					// add the script when opening the admin widget section
					add_action('admin_enqueue_scripts', array(&$this, 'load_widget_script') );
					
					// set the action for adding/removing sidebar via ajax
					if( current_user_can('manage_options') ){
						add_action('wp_ajax_gdlr_core_add_sidebar', array(&$this, 'add_sidebar'));	
						add_action('wp_ajax_gdlr_core_remove_sidebar', array(&$this, 'remove_sidebar'));	
					}
				}
								
			}
			
			// register sidebar to use in widget area
			function register_sidebar_widget(){

				$footer_args = apply_filters('gdlr_core_footer_widget_args', $this->sidebar_args);
				$sidebar_args = apply_filters('gdlr_core_sidebar_widget_args', $this->sidebar_args);	
	
				$sidebar_args['name'] = __('Custom Blog Sidebar', 'goodlayers-core');
				$sidebar_args['id'] = 'gdlr-core-sidebar-preset';
				$sidebar_args['description'] = esc_html__('Sidebar for blog posts', 'goodlayers-core');
				register_sidebar($sidebar_args);

				// widget for footer section
				foreach ( $this->footer_widgets as $widget ){
					$footer_args['name'] = esc_html($widget['name']);
					$footer_args['id'] = sanitize_title($widget['id']);
					$footer_args['description'] = empty($widget['description'])? '': $widget['description'];

					register_sidebar($footer_args);
				}
				
				// widget for content section
				$sidebar_args['class'] = 'gdlr-core-dynamic-widget';
				foreach ( $this->sidebars as $sidebar ){
					$sidebar_args['name'] = esc_html($sidebar['name']);
					$sidebar_args['id'] = sanitize_title($sidebar['id']);
					$sidebar_args['description'] = esc_html__('Custom widget area', 'goodlayers-core');

					register_sidebar($sidebar_args);
				}
				
			}
			
			// load the necessary script for the sidebar creator item
			function load_widget_script( $hook ){

				if( $hook == 'widgets.php' ){
					
					// include utility script
					gdlr_core_include_utility_script();

					// colorpicker
					wp_enqueue_style('wp-color-picker');
					wp_enqueue_script('wp-color-picker');
					
					// include the sidebar generator style
					wp_enqueue_style('gdlr-core-sidebar-generator', GDLR_CORE_URL . '/framework/css/sidebar-generator.css');
				
					// include the sidebar generator script
					wp_enqueue_script('gdlr-core-sidebar-generator', GDLR_CORE_URL . '/framework/js/sidebar-generator.js', array('jquery'), false, true);
					wp_localize_script('gdlr-core-sidebar-generator', 'gdlr_core_sidebar_generator', array(
						'ajaxurl' => GDLR_CORE_AJAX_URL,
						'error_head' => esc_html__('An error occurs', 'goodlayers-core'),
						'error_message' => esc_html__('Please refresh the page to try again. If the problem still persists, please contact administrator for this.', 'goodlayers-core'),
						'nonce' => wp_create_nonce('gdlr-core-sidebar-generator-nonce'),
						'title_text' => esc_html__('Create New Widget Area', 'goodlayers-core'),
						'add_new_text' => esc_html__('Add', 'goodlayers-core'),
					));	
					
				}
				
			}
			
			// add new sidebar to the array
			function gdlr_generate_sidebar_id( $sidebar_name ){
				
				// change sidebar name to valid html id
				$sidebar_id = gdlr_core_string_to_slug($sidebar_name);
				
				// assign id if empty
				$sidebar_id = empty($sidebar_id)? 'gdlr-core-sidebar': $sidebar_id;
				
				// check if id is already existed
				$sidebar_count = 0;
				$sidebar_id_addition = '';
				while( $sidebar_count < sizeOf($this->sidebars) ){
					foreach( $this->sidebars as $sidebar ){
						if( $sidebar['id'] != $sidebar_id . $sidebar_id_addition ){
							$sidebar_count++;
						}else{
							$sidebar_count = 0; 
							$sidebar_id_addition = empty($sidebar_id_addition)? 1: $sidebar_id_addition+1;
							break;
						}
					}
				}
				
				return $sidebar_id . $sidebar_id_addition;
				
			}
			
			// add new sidebar ajax module
			function add_sidebar(){
				
				if( !check_ajax_referer('gdlr-core-sidebar-generator-nonce', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}
				
				if( !empty($_POST['sidebar_name']) ){
					
					$sidebar_name = sanitize_text_field($_POST['sidebar_name']);
					$sidebar_id = $this->gdlr_generate_sidebar_id($sidebar_name);
					$this->sidebars[] = array(
						'name'=>$sidebar_name, 
						'id'=>$sidebar_id
					);
					
					if( update_option($this->sidebar_option_name, $this->sidebars) ){
						$ret = array('status'=> 'success');		
					}else{
						$ret = array(
							'status'=> 'failed',
							'head'=> esc_html__('Save Failed', 'goodlayers-core'),
							'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
						);						
					}
				}else{
					$ret = array(
						'status'=>'failed', 
						'head'=> esc_html__('An Error Occurs', 'goodlayers-core'),
						'message'=> esc_html__('Please assign the sidebar name and try again.' ,'goodlayers-core')
					);	
				}
				
				die(json_encode($ret));
				
			}	

			// remove sidebar ajax module
			function remove_sidebar(){
				
				if( !check_ajax_referer('gdlr-core-sidebar-generator-nonce', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}				
				
				if( isset($_POST['sidebar_id']) ){		
					
					$sidebar_id = sanitize_text_field($_POST['sidebar_id']);
					foreach( $this->sidebars as $key => $sidebar ){
						if( $sidebar['id'] == $sidebar_id ){
							unset($this->sidebars[$key]);
							break;
						}
					}
					
					if( update_option($this->sidebar_option_name, $this->sidebars) ){
						$ret = array('status'=> 'success');		
					}else{
						$ret = array(
							'status'=> 'failed',
							'head'=> esc_html__('Operation Failed', 'goodlayers-core'),
							'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
						);						
					}
				}else{
					$ret = array(
						'status'=>'failed',
						'head'=> esc_html__('Cannot Retrieve Sidebar Name', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					);	
				}
				
				die(json_encode($ret));
				
			}


		} // gdlr_core_sidebar_generator
		
	} // class_exists