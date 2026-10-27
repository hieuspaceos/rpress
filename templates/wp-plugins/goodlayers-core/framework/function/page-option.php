<?php
	/*	
	*	Goodlayers Page Builder
	*	---------------------------------------------------------------------
	*	File which creates page builder elements
	*	---------------------------------------------------------------------
	*/
	
	if( !class_exists('gdlr_core_page_option') ){
		
		class gdlr_core_page_option{
			
			// creating object
			private $settings = array();
			
			function __construct( $settings = array() ){
				
				$this->settings = wp_parse_args($settings, array(
					'title' => esc_html__('Page Options', 'goodlayers-core'),
					'slug' => 'gdlr-core-page-option',
					'post_type' => array('page'),
					'options' => array()
				));	
				
				// create custom meta box
				add_action('add_meta_boxes', array(&$this, 'init_page_option_meta_box'));
				
				// save custom metabox
				foreach( $this->settings['post_type'] as $post_type ){
					add_action('save_post_' . $post_type , array(&$this, 'save_page_option_meta_box'));
				}

				// ajax save
				add_action('wp_ajax_gdlr_core_save_page_option_data_' . implode('_', $this->settings['post_type']), array(&$this, 'ajax_save_page_option'));
				
				// add the script when opening the registered post type
				add_action('admin_enqueue_scripts', array(&$this, 'load_page_option_script') );
				
				// add page builder meta to revision
				gdlr_core_revision::add_field(array(
					'meta_key'=>$this->settings['slug'], 
					'meta_name'=>$this->settings['title'],
					'callback'=>array(&$this, 'convert_page_option_revision_data')
				));
			}
			
			// function that enqueue page builder script
			function load_page_option_script( $hook ){
				if( in_array($hook, array('post.php', 'post-new.php')) && in_array(get_post_type(), $this->settings['post_type']) ){
					gdlr_core_html_option::include_script(array(
						'style' => 'html-option-small'
					));

					$use_font_icons = apply_filters('gdlr_core_use_font_icons', array('font-awesome', 'elegant-font'));
					foreach($use_font_icons as $use_font_icon){
						gdlr_core_include_icon_font($use_font_icon);
					}
				}
			}
			
			// function that creats page builder meta box
			function init_page_option_meta_box(){
				
				foreach( $this->settings['post_type'] as $post_type ){
					add_meta_box($this->settings['slug'], $this->settings['title'],
						array(&$this, 'create_page_option_meta_box'),
						$post_type, 'normal', 'high' );	
				}
			}
			function create_page_option_meta_box( $post ){
				
				$page_option_value = get_post_meta($post->ID, $this->settings['slug'], true);

				// add nonce field to validate upon saving
				wp_nonce_field('gdlr_core_page_option', 'page_option_security');
				echo '<input type="hidden" class="gdlr-core-page-option-value" name="' . esc_attr($this->settings['slug']). '" value="' . esc_attr(json_encode($page_option_value)) . '" />';
				
				$this->get_option_head();

				echo '<div class="gdlr-core-page-option-content" >';
				$this->get_option_tab($page_option_value, $post);
				echo '</div>';
			}

			// page option head
			function get_option_head(){

				echo '<div class="gdlr-core-page-option-head" >';
				echo '<div class="gdlr-core-page-option-head-title">';
				echo '<i class="fa fa-gears"></i>';
				echo $this->settings['title'];
				echo '</div>'; // gdlr-core-page-option-head-title

				echo '<div class="gdlr-core-page-option-head-save" data-post-id="' . esc_attr(get_the_ID()) . '" ';
				echo 'data-ajax-url="' . esc_url(GDLR_CORE_AJAX_URL) . '" ';
				echo 'data-ajax-action="gdlr_core_save_page_option_data_' . esc_attr(implode('_', $this->settings['post_type'])) . '" ';
				echo 'data-failed-head="' . esc_attr__('An error occurs', 'goodlayers-core') . '" ';
				echo 'data-failed-message="' . esc_attr__('Please use wordpress update button to update the page instead.' ,'goodlayers-core') . '" ';
				echo ' >';
				echo '<i class="fa fa-save"></i>';
				echo esc_html__('Save Section', 'goodlayers-core');
				echo '</div>';
				echo '</div>';

			}

			// page option tab
			function get_option_tab($option_value, $post = null){

				$active = true;
				echo '<div class="gdlr-core-page-option-tab-head" id="gdlr-core-page-option-tab-head" >';
				foreach( $this->settings['options'] as $tab_slug => $tab_options ){
					echo '<div class="gdlr-core-page-option-tab-head-item ' . ($active? 'gdlr-core-active': '') . '" data-tab-slug="' . esc_attr($tab_slug) . '" >';
					echo gdlr_core_escape_content($tab_options['title']);
					echo '</div>'; // gdlr-core-page-option-tab-head-item

					$active = false;
				}
				echo '</div>'; // gdlr-core-page-option-tab-head

				$active = true;
				echo '<div class="gdlr-core-page-option-tab-content" id="gdlr-core-page-option-tab-content" >';
				foreach( $this->settings['options'] as $tab_slug => $tab_options ){
					echo '<div class="gdlr-core-page-option-tab-content-item ' . ($active? 'gdlr-core-active': '') . '" data-tab-slug="' . esc_attr($tab_slug) . '" >';
					foreach( $tab_options['options'] as $option_slug => $option ){
						$option['slug'] = $option_slug;
						if( !empty($option['single']) ){
							$option['value'] = get_post_meta($post->ID, $option['single'], true);
						}else if( isset($option_value[$option_slug]) ){
							$option['value'] = $option_value[$option_slug];
						}
						
						echo gdlr_core_html_option::get_element($option);
					}
					echo '</div>';
					
					$active = false;
				}
				echo '</div>'; // gdlr-core-page-option-tab-content
				
			}
			
			// ajax save meta box
			function ajax_save_page_option(){

				if( !check_ajax_referer('gdlr_core_page_option', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please use wordpress update button to update the page instead.' ,'goodlayers-core')
					)));
				}

				if( !empty($_POST['name']) && $_POST['name'] == $this->settings['slug'] ){
					if( !empty($_POST['post_id']) ){
						$value = json_decode(gdlr_core_process_post_data($_POST['value']), true);
						$value = wp_slash($value);
						if( !empty($value) ){
							foreach( $this->settings['options'] as $tab ){
								foreach( $tab['options'] as $option_slug => $option ){
									if( !empty($option['single']) && isset($value[$option_slug]) ){
										update_post_meta($_POST['post_id'], $option['single'], $value[$option_slug]);
										unset($value[$option_slug]);
									}
								}
							}
							
							update_post_meta($_POST['post_id'], $_POST['name'], $value);

							do_action('gdlr_core_after_update_page_option', $_POST['post_id']);
						}

						die(json_encode(array(
							'status' => 'success',
							'head' => esc_html__('Successfully Save', 'goodlayers-core'),
							'message'=> ''
						)));
					}

					die(json_encode(array(
						'status' => 'success',
						'head' => esc_html__('An error occurs', 'goodlayers-core'),
						'message' =>  esc_html__('Please use wordpress update button to update the page instead.' ,'goodlayers-core')
					)));

				}

			}

			// save post
			function save_page_option_meta_box( $post_id ){

				// check if nonce is available
				if( !isset($_POST['page_option_security']) ){
					return;
				}

				// vertify that the nonce is vaild
				if( !wp_verify_nonce($_POST['page_option_security'], 'gdlr_core_page_option') ) {
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
				if( !empty($_POST[$this->settings['slug']]) ){
					$value = json_decode(gdlr_core_process_post_data($_POST[$this->settings['slug']]), true);
					$value = wp_slash($value);
					foreach( $this->settings['options'] as $tab ){
						foreach( $tab['options'] as $option_slug => $option ){
							if( !empty($option['single']) && isset($value[$option_slug]) ){
								update_post_meta($post_id, $option['single'], $value[$option_slug]);
								unset($value[$option_slug]);
							}
						}
					}
					update_post_meta($post_id, $this->settings['slug'], $value);

					do_action('gdlr_core_after_update_page_option', $post_id);
				}

				// update revision auto num
				$auto_num = get_post_meta($post_id, 'gdlr-core-revision-auto-num', true);
				update_post_meta($post_id, 'gdlr-core-revision-auto-num', intval($auto_num) + 1);
			}
			
			// convert the data to read able revision format
			function convert_page_option_revision_data( $data ){
				return json_encode($data) . "\n";
			}

		} // gdlr_core_page_option
		
	} // class_exists